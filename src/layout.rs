mod config;
pub(crate) mod ops;

use crate::{
    Ramify, TryRamify,
    writer::{DiagramWrite, Style},
};

pub use self::config::Config;
pub(crate) use crate::columns::RowState;
use crate::columns::{Columns, SuspendedColumns};

/// A generator which incrementally writes the branch diagram to a writer.
///
/// Once you have a [`Ramify`] impementation, initialize this struct with the [`new`](Self::new) method or from [layout configuration](Config). After initializing, the branch
/// diagram can be incrementally written to a [diagram writer](DiagramWrite) using the
/// [`write_vertex`](Self::write_vertex) method.
///
/// ## Layout and style configuration
///
/// This struct can be configured by passing an appropriate [`Config`] struct. This is *layout*
/// configuration.
///
/// It is possible to modify configuration while writing the diagram (that is, in between calls to
/// [`write_vertex`](Self::write_vertex)) by using [`set_config`](Self::set_config).
///
/// ## Interaction with the [`Ramify`] trait.
///
/// ### Method call guarantees
///
/// When a [`Ramify`] implementation is used by a [`Generator`], the following calls are made
/// when rendering a row and its annotation (a single call to
/// [`write_vertex`](Generator::write_vertex)).
///
/// - [`Ramify::marker`] is called exactly once to determine the diagram marker for the minimal vertex.
/// - [`Ramify::annotate`] is called exactly once called to determine the annotation for the
///   minimal vertex.
/// - [`Ramify::ramify`] is called exactly once to replace the current minimal vertex with its
///   children
/// - [`Ramify::sort_key`] is called once for every active vertex every time a new vertex is
///   generated.
///
/// Moreover, the call to [`Ramify::ramify`] is **guaranteed to be last** for each vertex.
/// The other methods only take a reference to the vertex rather than receive the vertex itself.
///
/// Otherwise, the relative order between these calls, and moreover the order relative to writes, is unspecified.
///
/// ### Resource management
///
/// The vertex type `V` can either be borrowed or owned. If you are iterating over an in-memory
/// recursive type like
/// ```
/// struct Vtx<T>(T, Vec<Vtx<T>>);
/// ```
/// or an equivalent flattened version, then `V` is probably a lightweight type like `&'t Vtx` or a
/// `usize` index.
///
/// If the vertices are loaded in a streaming fashion, then most likely `V` is an owned type and
/// therefore it is managed by the generator.
///
/// Internally, the generator maintains a list of *active vertices*: the vertices not yet drawn to
/// the diagram, but for which a parent has already been drawn to the diagram. Once a vertex has
/// been drawn to the diagram, it is passed to [`Ramify::ramify`] or [`TryRamify::try_ramify`],
/// which takes ownership of `V`.
///
/// You can recover the active vertices using [`into_active_vertices`](Self::into_active_vertices).
///
/// ### Runtime and memory complexity
///
/// The branch diagram generator holds the minimal possible state required to generate the diagram.
/// This state is essentially the active vertices plus additional metadata concerning the column to which the vertex belongs in the diagram and whether the vertex is minimal.
/// More precisely, the memory usage is `(8 + size_of<V>) * num_active_vertices`,
/// plus the maximum size of a single annotation, plus a constant.
///
/// Writing a branch diagram row only requires making a finite number of passes over the list of vertices.
/// Therefore the runtime to write a single branch diagram row is `O(num_active_vertices)`,
/// assuming the various methods in [`Ramify`] take constant time.
///
/// If an annotation is written, the entire annotation is loaded into a scratch buffer. The scratch
/// buffer is re-used between calls to [`write_vertex`](Self::write_vertex).
#[derive(Debug)]
pub struct Generator<V, R> {
    columns: Columns<V, R>,
    annotation_buf: String,
    // in inverted mode, we need to avoid writing lines below the root
    first: bool,
}

impl<V, R> Generator<V, R> {
    /// Get a new branch diagram generator starting at a given vertex of type `V` using default
    /// configuration.
    pub fn new(root: V, ramifier: R) -> Self {
        Self::with_config(root, ramifier, Config::new())
    }

    /// Get a new branch diagram generator starting at a given vertex of type `V` using the provided
    /// configuration.
    pub fn with_config(root: V, ramifier: R, config: Config) -> Self {
        Self {
            columns: Columns::init(root, ramifier, config),
            annotation_buf: String::new(),
            first: true,
        }
    }

    /// Get a new branch diagram generator starting at a given vertex of type `V` using the default
    /// configuration.
    pub fn with_default_config(root: V, ramifier: R) -> Self {
        Self::with_config(root, ramifier, Config::new())
    }

    /// Returns the current configuration.
    pub fn config(&self) -> Config {
        self.columns.config()
    }

    /// Reset the configuration.
    pub fn set_config(&mut self, config: Config) {
        *self.columns.config_mut() = config;
    }

    /// The index of the final `open` edge, or `None` if there are no edges.
    ///
    /// For example, the below diagram has maximum edge index `4`.
    /// ```txt
    /// 0
    /// ├┬╮
    /// │1│
    /// ├╮╰─╮
    /// ```
    /// This is not the same as the width of the diagram row which was previously written. However,
    /// we can use this information to compute the width of the diagram row by taking the maximum of the edge index and the
    /// edge index prior to writing a row, multiplying by the gutter width, and then adding `1`.
    pub fn max_edge_index(&self) -> Option<usize> {
        self.columns.max_edge_index()
    }

    /// The number of active vertices.
    ///
    /// An active vertex is a vertex which has not yet been written to the branch diagram, but
    /// whose parent was already written. Since multiple vertices may use the same edge, this
    /// number is distinct from the number of outgoing edges.
    ///
    /// The count will include equivalent vertices, excluding those that are equivalent to the
    /// current minimal vertex.
    pub fn num_active_vertices(&self) -> usize {
        self.columns.girth()
    }

    /// Whether or not there are any active vertices.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Consume the generator, returning an iterator over the active vertices in an unspecified
    /// order.
    pub fn into_active_vertices(self) -> impl ExactSizeIterator<Item = V> {
        self.columns.into_active_vertices()
    }

    /// Shrink the capacity of internal allocations as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.annotation_buf.shrink_to_fit();
        self.columns.shrink_to_fit();
    }

    /// Returns the annotation written in the previous vertex, if any.
    ///
    /// This returns the empty string if there was no previous vertex or if the previous vertex
    /// did not have an annotation.
    pub fn previous_annotation(&self) -> &str {
        &self.annotation_buf
    }
}

impl<V, R> Generator<V, R> {
    /// Write a row containing a vertex along with its annotation to the provided
    /// [diagram writer](DiagramWrite).
    ///
    /// This method takes ownership since a write error leaves the generator in an unspecified
    /// state from which resuming generation is not possible.
    ///
    /// # Output rows
    ///
    /// A single call to this method writes the following:
    ///
    /// 1. The annotation lines (if any), with the vertex on the first or last line depending on
    ///    the [configuration](Config).
    /// 2. Rows for the row padding (if not last).
    /// 3. Any extra rows to prepare for the next vertex (if not last). This includes merge lines,
    ///    if merges are required.
    pub fn write_vertex<W: DiagramWrite>(self, writer: &mut W) -> Result<Self, W::Error>
    where
        R: Ramify<V>,
    {
        let State::Ok(generator) = self.try_write_vertex(writer)?;
        Ok(generator)
    }

    /// Write the entire branch diagram into the provided writer.
    ///
    /// This repeatedly calls [`write_vertex`](Generator::write_vertex) as long as the there are
    /// remaining vertices. The (empty) generator is returned at the end.
    pub fn write_all_vertices<W: DiagramWrite>(mut self, writer: &mut W) -> Result<Self, W::Error>
    where
        R: Ramify<V>,
    {
        while !self.is_empty() {
            self = self.write_vertex(writer)?;
        }
        Ok(self)
    }

    /// Generate the entire branch diagram as a newly allocated string.
    ///
    /// This is identical to repeatedly calling [`write_vertex`](Self::write_vertex) as long as the
    /// generator is [is not empty](Generator::is_empty) with a
    /// [`FmtWriter`](crate::writer::FmtWriter) wrapping a string buffer.
    pub fn branch_diagram(mut self, style: Style) -> String
    where
        R: Ramify<V>,
    {
        let mut buf = String::new();
        let mut writer = style.fmt_writer(&mut buf);
        while !self.is_empty() {
            self = self
                .write_vertex(&mut writer)
                .expect("Failed to write into string");
        }

        buf
    }
}

impl<V, R: TryRamify<V>> Generator<V, R> {
    /// Try to write the next vertex, failing to do so if the call to [`TryRamify::try_ramify`]
    /// results in an error.
    ///
    /// The error is handled *eagerly*: no writes are performed when an error is encountered. This
    /// puts the generator into a [suspended state](SuspendedGenerator), and iteration can be
    /// continued by supplying a (possibly empty) list of children.
    ///
    /// If the generator is [empty](Self::is_empty), this does nothing.
    pub fn try_write_vertex<W: DiagramWrite>(
        mut self,
        writer: &mut W,
    ) -> Result<State<V, R, R::Error>, W::Error> {
        match self.columns.try_substitute(&mut self.annotation_buf) {
            Ok((g, None)) => {
                self.columns = g;
                Ok(State::Ok(self))
            }
            Ok((mut g, Some((col, marker_char)))) => {
                try_write_vertex_impl(
                    &mut g,
                    col,
                    marker_char,
                    &self.annotation_buf,
                    writer,
                    self.first,
                )?;
                self.first = false;
                self.columns = g;
                Ok(State::Ok(self))
            }
            Err((f, err)) => {
                let failed = SuspendedGenerator {
                    columns: f,
                    annotation_buf: self.annotation_buf,
                    first: self.first,
                };
                Ok(State::Suspended(failed, err))
            }
        }
    }
}

/// The possible generator states which may occur after a call to
/// [`try_write_vertex`](Generator::try_write_vertex).
pub enum State<V, R, E> {
    /// The vertex was written successfully.
    Ok(Generator<V, R>),
    /// The vertex was not written because of an error.
    Suspended(SuspendedGenerator<V, R>, E),
}

impl<V, R, E> State<V, R, E> {
    /// Return the generator if ok, or drop the suspended generator and return the error if not.
    pub fn halt_if_suspended(self) -> Result<Generator<V, R>, E> {
        match self {
            Self::Ok(generator) => Ok(generator),
            Self::Suspended(_, err) => Err(err),
        }
    }

    /// A convenience function to either write the next vertex or resume from the suspended state
    /// with a closure.
    ///
    /// If this is a [`State::Ok`], this calls [`Generator::try_write_vertex`], and if this is a
    /// [`State::Suspended`], the provided closure is applied to the error to provide a new list of
    /// children.
    pub fn try_write_vertex<I, F, W>(
        self,
        writer: &mut W,
        f: F,
    ) -> Result<State<V, R, R::Error>, W::Error>
    where
        R: TryRamify<V>,
        I: IntoIterator<Item = V>,
        F: FnOnce(E) -> I,
        W: DiagramWrite,
    {
        match self {
            Self::Ok(generator) => generator.try_write_vertex(writer),
            Self::Suspended(suspended, err) => {
                let generator = suspended.resume(writer, f(err))?;
                Ok(State::Ok(generator))
            }
        }
    }

    /// Whether or not there are any active vertices.
    ///
    /// If this is a [`State::Ok`], this checks if the internal iterator is non-empty, and if this
    /// is [`State::Suspended`] it always returns false since there is at least one active
    /// vertex.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Ok(generator) => generator.is_empty(),
            Self::Suspended(_, _) => false,
        }
    }

    /// Consume the state, returning an iterator over the active vertices
    /// in an unspecified order.
    pub fn into_active_vertices(self) -> impl ExactSizeIterator<Item = V> {
        match self {
            Self::Ok(generator) => generator.columns.into_active_vertices(),
            Self::Suspended(suspended, _) => suspended.columns.into_active_vertices(),
        }
    }
}

/// A suspended generator.
///
/// A suspended generator is like a normal generator, except with the minimal vertex moved out of
/// the generator. This state results when a [`TryRamify`] implementation returns an error.
///
/// Iteration can be [resumed](Self::resume), which requires specifying an iterator over children
/// manually.
pub struct SuspendedGenerator<V, R> {
    columns: SuspendedColumns<V, R>,
    annotation_buf: String,
    first: bool,
}

impl<V, R> SuspendedGenerator<V, R> {
    /// Recover from an error and resume iteration.
    ///
    /// This writes the current minimal vertex and replaces it with its children. The end result is
    /// equivalent to the [`TryRamify`] implementation succeeding and yielding this iterator over
    /// children.
    ///
    /// In order to resume iteration, the caller must provide children for the failed vertex. The
    /// children are used to write a single vertex row, and then the resulting generator is
    /// returned.
    pub fn resume<I, W: DiagramWrite>(
        self,
        writer: &mut W,
        children: I,
    ) -> Result<Generator<V, R>, W::Error>
    where
        R: TryRamify<V>,
        I: IntoIterator<Item = V>,
    {
        let (mut g, col, marker_char) = self.columns.resume(children);

        try_write_vertex_impl(
            &mut g,
            col,
            marker_char,
            &self.annotation_buf,
            writer,
            self.first,
        )?;

        Ok(Generator {
            columns: g,
            first: false,
            annotation_buf: self.annotation_buf,
        })
    }

    /// Consume the suspended generator, returning an iterator over the active vertices
    /// in an unspecified order.
    pub fn into_active_vertices(self) -> impl ExactSizeIterator<Item = V> {
        self.columns.into_active_vertices()
    }

    /// Returns the annotation that will be written with the next vertex if iteration is resumed.
    pub fn next_annotation(&self) -> &str {
        &self.annotation_buf
    }
}

/// Write a row which prepares for the vertex to be written.
///
/// This does the following:
///
/// 1. Make all of the vertices isolated.
/// 2. Once isolated, merges the vertices if needed.
fn write_preparation_row<W: DiagramWrite, V, R: TryRamify<V>>(
    cols: &mut Columns<V, R>,
    writer: &mut W,
    state: &mut RowState,
) -> Result<(), W::Error> {
    let new = if state.is_isolated() && !cols.is_merged() {
        cols.write_merge_row(writer, ops::Merge)?
    } else {
        cols.write_row(writer, ops::Fork)?
    };
    state.update(&new);
    Ok(())
}

fn write_preparation_row_delayed<W: DiagramWrite, V, R: TryRamify<V>>(
    cols: &mut Columns<V, R>,
    writer: &mut W,
    first: bool,
    col: &mut usize,
) -> Result<RowState, W::Error> {
    if first {
        cols.write_row(writer, ops::Skip)
    } else {
        cols.write_shimmed_row(writer, ops::Fork, (*col, ops::DelayedFork(col)))
    }
}

fn write_vertex_row<W: DiagramWrite, V, R: TryRamify<V>>(
    cols: &mut Columns<V, R>,
    writer: &mut W,
    col: usize,
    marker_char: char,
) -> Result<RowState, W::Error> {
    cols.write_shimmed_row(writer, ops::Fork, (col, ops::Marker(marker_char)))
}

/// Internal implementation
// impl<V, R: TryRamify<V>> Generator<V, R> {
fn try_write_vertex_impl<V, R: TryRamify<V>, W: DiagramWrite>(
    columns: &mut Columns<V, R>,
    col: usize,
    marker_char: char,
    annotation: &str,
    writer: &mut W,
    first: bool,
) -> Result<(), W::Error> {
    match (
        columns.config().annotation_before_vertex,
        columns.config().reverse_annotation_lines,
    ) {
        (false, false) => {
            try_write_vertex_normal_impl(columns, col, marker_char, annotation.lines(), writer)
        }
        (false, true) => try_write_vertex_normal_impl(
            columns,
            col,
            marker_char,
            annotation.lines().rev(),
            writer,
        ),
        (true, false) => try_write_vertex_delayed_impl(
            columns,
            col,
            marker_char,
            annotation.lines(),
            writer,
            first,
        ),
        (true, true) => try_write_vertex_delayed_impl(
            columns,
            col,
            marker_char,
            annotation.lines().rev(),
            writer,
            first,
        ),
    }
}

fn try_write_vertex_normal_impl<'a, V, R: TryRamify<V>, W: DiagramWrite>(
    columns: &mut Columns<V, R>,
    col: usize,
    marker_char: char,
    lines: impl Iterator<Item = &'a str>,
    writer: &mut W,
) -> Result<(), W::Error> {
    // write the vertex row and get the diagram width
    let mut state = write_vertex_row(columns, writer, col, marker_char)?;

    let mut lines = lines.enumerate();

    // finish the vertex row and then write the annotation lines
    match lines.next() {
        Some((idx, first_line)) => {
            writer.write_annotation_line(idx, state.width, state.alignment, first_line)?;

            // write the remaining annotation lines
            for (idx, line) in lines {
                write_preparation_row(columns, writer, &mut state)?;
                writer.write_annotation_line(idx, state.width, state.alignment, line)?;
            }
        }
        None => writer.write_newline()?,
    }

    // prepare for the next row, writing at least enough rows to get the desired
    // padding (except on the last row)
    if !columns.is_empty() {
        let mut padding = columns.config().row_padding;
        while padding > 0 {
            write_preparation_row(columns, writer, &mut state)?;
            writer.write_newline()?;
            padding -= 1;
        }
        while !state.is_ready() {
            write_preparation_row(columns, writer, &mut state)?;
            writer.write_newline()?;
        }
    }
    Ok(())
}

/// Write the vertex at the end of the annotation instead of at the beginning.
fn try_write_vertex_delayed_impl<'a, V, R: TryRamify<V>, W: DiagramWrite>(
    columns: &mut Columns<V, R>,
    mut col: usize,
    marker_char: char,
    mut lines: impl DoubleEndedIterator<Item = &'a str>,
    writer: &mut W,
    first: bool,
) -> Result<(), W::Error> {
    let maybe_last_line = lines.next_back();

    // we cannot use `enumerate` because we don't have an exact size iterator, so we manually
    // implement it since we only write the last line at the end anyway.
    let mut idx = 0;

    let mut state = match maybe_last_line {
        None => {
            // no annotation, so we can already prepare for the next row
            let state = write_vertex_row(columns, writer, col, marker_char)?;

            writer.write_newline()?;
            state
        }
        Some(last_line) => {
            match lines.next() {
                None => {
                    let state = columns.write_shimmed_row(
                        writer,
                        ops::Fork,
                        (col, ops::DelayedMarker(marker_char)),
                    )?;
                    writer.write_annotation_line(idx, state.width, state.alignment, last_line)?;

                    state
                }
                Some(first_line) => {
                    let mut state =
                        write_preparation_row_delayed(columns, writer, first, &mut col)?;
                    writer.write_annotation_line(idx, state.width, state.alignment, first_line)?;
                    idx += 1;

                    for line in lines {
                        let new = write_preparation_row_delayed(columns, writer, first, &mut col)?;
                        state.update(&new);
                        writer.write_annotation_line(idx, state.width, state.alignment, line)?;
                        idx += 1;
                    }

                    let new_state = columns.write_shimmed_row(
                        writer,
                        ops::Fork,
                        (col, ops::DelayedMarker(marker_char)),
                    )?;

                    // temporarily store the width, etc. in the previous state, and use it
                    // to write the annotation
                    state.update(&new_state);
                    writer.write_annotation_line(idx, state.width, state.alignment, last_line)?;

                    new_state
                }
            }
        }
    };

    // write the padding and prepare for the next vertex (unless this is the last row)
    if !columns.is_empty() {
        let mut padding = columns.config().row_padding;
        while padding > 0 {
            write_preparation_row(columns, writer, &mut state)?;
            writer.write_newline()?;
            padding -= 1;
        }

        // make the minimal index a singleton so that the vertex row can be written.
        while !state.is_ready() {
            write_preparation_row(columns, writer, &mut state)?;
            writer.write_newline()?;
        }
    }
    Ok(())
}
