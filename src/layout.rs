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
/// [`write`](Self::write) or [`try_write`](Self::try_write) methods.
///
/// ## Layout and style configuration
///
/// This struct can be configured by passing an appropriate [`Config`] struct. This is *layout*
/// configuration. The appearance of the resulting diagram is defined by the diagram writer. The
/// build in diagram writers ([`IOWriter`](crate::writer::IOWriter) and
/// [`FmtWriter`](crate::writer::FmtWriter)) use the [`Style`] struct for configuration.
///
/// It is possible to modify configuration while writing the diagram (that is, in between calls to
/// [`write`](Self::write)) by using [`set_config`](Self::set_config).
///
/// ## Interaction with the [`Ramify`] trait.
///
/// ### Method call guarantees
///
/// When a [`Ramify`] implementation is used by a [`Generator`], the following calls are made
/// when rendering a row and its annotation (a single call to
/// [`write`](Generator::write)).
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
/// buffer is re-used between calls to [`write`](Self::write).
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

    /// Write a row containing a vertex along with its annotation to the provided
    /// [diagram writer](DiagramWrite).
    ///
    /// This method takes ownership since a write error leaves the generator in an unspecified
    /// state from which resuming generation is not possible.
    ///
    /// If the generator is [empty](Self::is_empty), this does nothing.
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
    pub fn write<W: DiagramWrite>(self, writer: &mut W) -> Result<Self, W::Error>
    where
        R: Ramify<V>,
    {
        let State::Ok(generator) = self.try_write(writer)?;
        Ok(generator)
    }

    /// Try to write the next vertex, failing to do so if the call to [`TryRamify::try_ramify`]
    /// results in an error.
    ///
    /// The error is handled *eagerly*: no writes are performed when an error is encountered. An
    /// error
    /// puts the generator into a [suspended state](SuspendedGenerator), and iteration can be
    /// continued by supplying a (possibly empty) list of children.
    ///
    /// If the generator is [empty](Self::is_empty), this does nothing.
    pub fn try_write<W: DiagramWrite>(
        mut self,
        writer: &mut W,
    ) -> Result<State<V, R, R::Error>, W::Error>
    where
        R: TryRamify<V>,
    {
        match self.columns.try_substitute(&mut self.annotation_buf) {
            Ok((g, None)) => {
                self.columns = g;
                Ok(State::Ok(self))
            }
            Ok((mut g, Some((col, marker_char)))) => {
                try_write_impl(
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

    /// Write the entire branch diagram into the provided diagram writer.
    ///
    /// This repeatedly calls [`write`](Generator::write) as long as the there are
    /// remaining vertices.
    pub fn write_all<W: DiagramWrite>(mut self, writer: &mut W) -> Result<(), W::Error>
    where
        R: Ramify<V>,
    {
        while !self.is_empty() {
            self = self.write(writer)?;
        }
        Ok(())
    }

    /// Try to write the entire branch diagram into the provided diagram writer.
    ///
    /// This repeatedly calls [`try_write`](Generator::write) as long as the writes succeed. If a
    /// write fails, the suspended generator is returned along with the error which occurred.
    #[expect(clippy::type_complexity)]
    pub fn try_write_all<W: DiagramWrite>(
        mut self,
        writer: &mut W,
    ) -> Result<Option<(SuspendedGenerator<V, R>, R::Error)>, W::Error>
    where
        R: TryRamify<V>,
    {
        while !self.is_empty() {
            self = match self.try_write(writer)? {
                State::Ok(generator) => generator,
                State::Suspended(suspended, err) => return Ok(Some((suspended, err))),
            };
        }
        Ok(None)
    }

    /// Generate the entire branch diagram as a newly allocated string.
    ///
    /// This is identical to calling [`write_all`](Self::write_all) with a
    /// [`FmtWriter`](crate::writer::FmtWriter) wrapping a string.
    pub fn branch_diagram(self, style: Style) -> String
    where
        R: Ramify<V>,
    {
        let mut buf = String::new();
        self.write_all(&mut style.fmt_writer(&mut buf))
            .expect("Failed to write into string!");
        buf
    }

    /// Get a copy of the current configuration.
    pub fn config(&self) -> Config {
        self.columns.config()
    }

    /// Set the configuration to a new value.
    pub fn set_config(&mut self, config: Config) {
        *self.columns.config_mut() = config;
    }

    /// The index of the largest occupied column, or `None` if the diagram is empty.
    ///
    /// For example, the below diagrams have maximum edge index `4` and `1` respectively.
    /// ```txt
    /// 0        0
    /// ├┬╮      ├┬╮
    /// │1│      │1│
    /// ├╮╰─╮    │╭╯
    ///     ^     ^
    ///     4     1
    /// ```
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

    /// The annotation of the previous vertex, if any.
    ///
    /// This returns the empty string if there was no previous vertex or if the previous vertex
    /// did not have an annotation.
    pub fn previous_annotation(&self) -> &str {
        &self.annotation_buf
    }
}

/// Generator states which may occur after a call to [`try_write`](Generator::try_write).
///
/// If you want to abort on an error, use [`halt_if_suspended`](Self::halt_if_suspended). If you
/// want to continue despite the error, use [`SuspendedGenerator::resume`].
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

    /// Either write the next vertex or resume from the suspended state with a closure.
    ///
    /// If this is a [`State::Ok`], this calls [`Generator::try_write`], and if this is a
    /// [`State::Suspended`], the provided closure is applied to the error to provide a new list of
    /// children.
    pub fn try_write<I, F, W>(self, writer: &mut W, f: F) -> Result<State<V, R, R::Error>, W::Error>
    where
        R: TryRamify<V>,
        I: IntoIterator<Item = V>,
        F: FnOnce(E) -> I,
        W: DiagramWrite,
    {
        match self {
            Self::Ok(generator) => generator.try_write(writer),
            Self::Suspended(suspended, err) => {
                let generator = suspended.resume(writer, f(err))?;
                Ok(State::Ok(generator))
            }
        }
    }

    /// A convenience function to repeatedly try to write the next vertex or resume from the suspended state with a closure.
    ///
    /// Note that instead of using this method, it may be possible to directly implement
    /// [`Ramify`] by inlining the closure into [`try_ramify`](TryRamify::try_ramify) and instead
    /// using [`Generator::write_all`].
    pub fn try_write_all<I, F, W>(mut self, writer: &mut W, mut f: F) -> Result<(), W::Error>
    where
        R: TryRamify<V, Error = E>,
        I: IntoIterator<Item = V>,
        F: FnMut(E) -> I,
        W: DiagramWrite,
    {
        while !self.is_empty() {
            self = self.try_write(writer, &mut f)?;
        }
        Ok(())
    }

    /// Whether or not there are any active vertices.
    ///
    /// If this is a [`State::Ok`], this checks if the generator is non-empty, and if this
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
/// A suspended generator is like a normal generator with the minimal vertex moved out of
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
    /// This writes the current minimal vertex and updates the internal state to hold the provided children.
    /// The end result is equivalent to the [`TryRamify`] implementation succeeding and yielding
    /// the provided children.
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

        try_write_impl(
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

    /// Returns the annotation that will be written if iteration is resumed.
    pub fn peek_annotation(&self) -> &str {
        &self.annotation_buf
    }
}

/// Write a row which prepares for the vertex to be written.
///
/// This does the following:
///
/// 1. Makes all of the vertices isolated.
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

#[inline]
fn write_annotation_line<W: DiagramWrite>(
    writer: &mut W,
    idx: usize,
    state: &RowState,
    // width: usize,
    // alignment: usize,
    line: &str,
) -> Result<(), W::Error> {
    writer.prepare_annotation(idx, state.width, state.alignment)?;
    writer.write_annotation(idx, line)?;
    writer.write_newline()
}

fn try_write_impl<V, R: TryRamify<V>, W: DiagramWrite>(
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
            try_write_normal_impl(columns, col, marker_char, annotation.lines(), writer)
        }
        (false, true) => {
            try_write_normal_impl(columns, col, marker_char, annotation.lines().rev(), writer)
        }
        (true, false) => {
            try_write_delayed_impl(columns, col, marker_char, annotation.lines(), writer, first)
        }
        (true, true) => try_write_delayed_impl(
            columns,
            col,
            marker_char,
            annotation.lines().rev(),
            writer,
            first,
        ),
    }
}

fn try_write_normal_impl<'a, V, R: TryRamify<V>, W: DiagramWrite>(
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
            write_annotation_line(writer, idx, &state, first_line)?;

            // write the remaining annotation lines
            for (idx, line) in lines {
                write_preparation_row(columns, writer, &mut state)?;
                write_annotation_line(writer, idx, &state, line)?;
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
fn try_write_delayed_impl<'a, V, R: TryRamify<V>, W: DiagramWrite>(
    columns: &mut Columns<V, R>,
    mut col: usize,
    marker_char: char,
    mut lines: impl DoubleEndedIterator<Item = &'a str>,
    writer: &mut W,
    first: bool,
) -> Result<(), W::Error> {
    let maybe_last_line = lines.next_back();

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
                    write_annotation_line(writer, 0, &state, last_line)?;

                    state
                }
                Some(first_line) => {
                    let mut state =
                        write_preparation_row_delayed(columns, writer, first, &mut col)?;
                    write_annotation_line(writer, 0, &state, first_line)?;

                    // we cannot use `enumerate` because we don't have an exact size iterator, so we manually
                    // implement it since we only write the last line at the end at which point we know its index
                    // anyway.
                    let mut idx = 1;

                    for line in lines {
                        let new = write_preparation_row_delayed(columns, writer, first, &mut col)?;
                        state.update(&new);
                        write_annotation_line(writer, idx, &state, line)?;
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
                    write_annotation_line(writer, idx, &state, last_line)?;

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
