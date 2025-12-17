pub(crate) mod ops;
#[cfg(test)]
pub(crate) mod tests;

use std::{convert::Infallible, io};

use crate::{
    Config, Ramify, TryRamify,
    writer::{
        DiagramWriter, DoubledLines, RoundedCorners, RoundedCornersWide, SharpCorners,
        SharpCornersWide, WriteBranch,
    },
};

use crate::columns::Columns;
pub(crate) use crate::columns::RowState;

/// A generator which incrementally writes the branch diagram to a writer.
///
/// Once you have a [`Ramify`] impementation, initialize this struct with the [`init`](Self::init) method. After initializing, the branch
/// diagram can be incrementally written to a [writer](io::Write) using the
/// [`write_vertex`](Self::write_vertex) method.
///
/// The documentation here is mostly relevant for using the [`Generator`]. The layout algorithm
/// is documented in the [`writer` module](crate::writer#layout-algorithm-documentation).
///
/// ## Compile-time and dynamic configuration
///
/// This struct can be configured by passing an appropriate [`Config`] struct. The configuration
/// contains compile-time and runtime configuration. The compile-time configuration is included in
/// the state parameter (for example, a [`RoundedCorners`] parameter), which describes the appearance of the
/// branch diagram. The runtime configuration concerns configuration relevant to the layout algorithm.
///
/// It is possible to modify configuration while writing the diagram (that is, in between calls to
/// [`write_vertex`](Self::write_vertex)) by using the [`config_mut`](Self::config_mut)
/// method. Any such modifications of the configuration are guaranteed to not
/// corrupt the branch diagram.
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
/// Moreover, the call to [`Ramify::ramify`] is **guaranteed to be last** for each vertex. This is enforced by the borrow checker since the signature takes ownership of `V`.
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
/// A generator will *never* silently drop a vertex while it is running. Therefore all of the resource
/// management takes place in the [`Ramify`] or [`TryRamify`] implementation. This occurs in two
/// places:
///
/// - When computing the children of a vertex, ownership is passed to the [`Ramify::ramify`]
///   function call.
/// - If the vertex is identical to the minimal vertex, it is passed to [`Ramify::cleanup`].
///
/// If you drop a generator, the active vertices will be de-allocated. You can recover the
/// active vertices using [`into_active_vertices`](Self::into_active_vertices).
///
/// ### Runtime and memory complexity
///
/// The branch diagram generator holds the minimal possible state required to generate the diagram.
/// This state is essentially the active vertices plus additional metadata concerning the column to which the vertex belongs in the diagram and whether the vertex is minimal.
/// More precisely, the memory usage is `(8 + size_of<V>) * num_active_vertices`,
/// plus the maximum size of a single annotation, plus a constant.
///
/// Writing a branch diagram row only requires making a fininte number of passes over the list of vertices.
/// Therefore the runtime to write a single branch diagram row is `O(num_active_vertices)`,
/// assuming the various methods in [`Ramify`] take constant time.
///
/// If an annotation is written, the entire annotation is loaded into a scratch buffer. The scratch
/// buffer is re-used between calls to [`write_vertex`](Self::write_vertex).
#[derive(Debug)]
pub struct Generator<V, R, B, P = Infallible> {
    columns: Columns<V, R, B, P>,
    annotation_buf: String,
    // in inverted mode, we need to avoid writing lines below the root
    first: bool,
}

impl<V, R, B: WriteBranch, P> Generator<V, R, B, P> {
    /// Get a new branch diagram generator starting at a given vertex of type `V` using the provided
    /// configuration.
    pub fn init(root: V, ramifier: R, config: Config<B>) -> Self {
        Self {
            columns: Columns::init(root, ramifier, config),
            annotation_buf: String::new(),
            first: true,
        }
    }

    /// Get a new branch diagram generator starting at a given vertex of type `V` using the default
    /// configuration.
    ///
    /// Calling this method requires type annotations. Also see the convenience methods:
    ///
    /// - [`with_rounded_corners`](Self::with_rounded_corners)
    /// - [`with_rounded_corners_wide`](Self::with_rounded_corners_wide)
    /// - [`with_sharp_corners`](Self::with_sharp_corners)
    /// - [`with_sharp_corners_wide`](Self::with_sharp_corners_wide)
    /// - [`with_doubled_lines`](Self::with_doubled_lines)
    pub fn with_default_config(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::new())
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &Config<B> {
        self.columns.config()
    }

    /// Returns a mutable reference to the configuration.
    ///
    /// The configuration parameters can be safely changed while writing the branch diagram.
    pub fn config_mut(&mut self) -> &mut Config<B> {
        self.columns.config_mut()
    }
}

/// An error which can occur when calling [`Generator::try_write_vertex`].
#[derive(Debug)]
pub enum WriteVertexError<E> {
    /// An IO error was raised by the writer.
    IO(io::Error),
    /// The [`TryRamify`] implementation failed to determine the children for the active vertex and
    /// returned the corresponding error.
    TryChildrenFailed(E),
}

impl<E> From<io::Error> for WriteVertexError<E> {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl<E: std::fmt::Display> std::fmt::Display for WriteVertexError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error occurred while writing vertex: ")?;
        match self {
            Self::IO(error) => error.fmt(f),
            Self::TryChildrenFailed(error) => error.fmt(f),
        }
    }
}

/// Write a row which prepares for the vertex to be written.
///
/// This does the following:
///
/// 1. Make all of the vertices isolated.
/// 2. Once isolated, merges the vertices if needed.
fn write_preparation_row<W: io::Write, V, R: TryRamify<V>, B: WriteBranch>(
    cols: &mut Columns<V, R, B, R::Placeholder>,
    writer: &mut DiagramWriter<W, B>,
    state: &mut RowState,
) -> io::Result<()> {
    let new = if state.is_isolated() && !cols.is_merged() {
        cols.write_merge_row(writer, ops::Merge)?
    } else {
        cols.write_row(writer, ops::Fork)?
    };
    state.update(&new);
    Ok(())
}

fn write_preparation_row_inverted<W: io::Write, V, R: TryRamify<V>, B: WriteBranch>(
    cols: &mut Columns<V, R, B, R::Placeholder>,
    writer: &mut DiagramWriter<W, B>,
    first: bool,
    col: &mut usize,
) -> io::Result<RowState> {
    if first {
        cols.write_row(writer, ops::Skip)
    } else {
        cols.write_shimmed_row(writer, ops::Fork, (*col, ops::Extra(col)))
    }
}

fn write_vertex_row<W: io::Write, V, R: TryRamify<V>, B: WriteBranch>(
    cols: &mut Columns<V, R, B, R::Placeholder>,
    writer: &mut DiagramWriter<W, B>,
    col: usize,
    marker_char: char,
) -> io::Result<RowState> {
    cols.write_shimmed_row(writer, ops::Fork, (col, ops::Marker(marker_char)))
}

impl<E: std::error::Error> std::error::Error for WriteVertexError<E> {}

impl<V, R, B: WriteBranch> Generator<V, R, B> {
    /// Write a row containing a vertex along with its annotation to the provided writer.
    ///
    /// This method returns `Ok(true)` if there are vertices remaining, and otherwise `Ok(false)`.
    ///
    /// # Output rows
    ///
    /// A single call to this method will write a row containing the vertex, along with other rows:
    ///
    /// - If not inverted, the vertex is written first, followed by a number of non-marker rows in order to accommodate
    ///   additional lines of annotation and to set the generator state so that the subsequent
    ///   call can immediately write a vertex.
    /// - If inverted, the vertex is prepared, and then the annotations are written with the vertex
    ///   on the final row. The vertex must be prepared before the annotations since it is not
    ///   known in advance how many rows will be required to write the vertex row.
    ///
    /// # Correctly rendering inverted mode
    ///
    /// In inverted mode, the annotation lines are written in reverse order and aligned so that the final
    /// annotation line coincides with the vertex. This makes the annotations look correct
    /// if the branch diagram is displayed with the root at the bottom. The most common way to do this is to
    /// first write the branch diagram into a string buffer, and then write the lines of the buffer
    /// in reverse.
    ///
    /// # Valid Unicode
    ///
    /// This method will only write valid Unicode bytes into the writer. For example, writing into
    /// a string buffer using [`String::as_mut_vec`] will not result in undefined behaviour. Also
    /// see [`write_vertex_str`](Self::write_vertex_str).
    ///
    /// # Buffered writes
    ///
    /// The implementation tries to minimize the number of [`write`](io::Write::write) made by this method,
    /// but the number of calls is still large. It is recommended that the provided writer is
    /// buffered, for example using an [`io::BufWriter`] or an [`io::LineWriter`]. Many writers
    /// provided by the standard library are already buffered.
    pub fn write_vertex<W: io::Write>(&mut self, writer: W) -> io::Result<bool>
    where
        R: Ramify<V>,
    {
        self.try_write_vertex(writer).map_err(|e| match e {
            WriteVertexError::IO(error) => error,
            // the implementation of TryRamify if `R` is `Ramify` always succeeds because
            // of the blanket implementation
            WriteVertexError::TryChildrenFailed(_) => unreachable!(),
        })
    }

    /// Write a row containing a vertex along with its annotation to the provided string buffer.
    ///
    /// This is identical to [`write_vertex`](Self::write_vertex), except there is no
    /// error since writing will not fail (unless you run out of memory).
    pub fn write_vertex_str(&mut self, buf: &mut String) -> bool
    where
        R: Ramify<V>,
    {
        self.write_vertex(unsafe { buf.as_mut_vec() })
            .expect("Out of memory!")
    }
}

impl<V, R: TryRamify<V>, B: WriteBranch> Generator<V, R, B, R::Placeholder> {
    /// Attempt to write the next vertex, failing to do so if the call to [`TryRamify::try_ramify`]
    /// results in an error.
    ///
    /// The error is propagated in the [`WriteVertexError`] and can be used
    /// by the caller to decide whether or not to attempt to write the row again.
    ///
    /// # Handling of the replacement vertex
    ///
    /// The replacement vertex will be used as the minimal vertex, even if the vertex changed. In
    /// most cases, you should just return the original vertex, but an alternative could be returned
    /// if the original vertex should not be attempted again.
    ///
    /// In normal rendering order, this means that no writes will occur when rendering fails. In
    /// inverted mode, any writes which prepare the vertex will still succeed, but will not be
    /// repeated on the next attempt. In either case, assuming the original vertex is returned as
    /// the replacement, rendering is *idempotent*: failing to obtain the children `n` times,
    /// followed by a success, is exactly the same succeeding on the first try.
    pub fn try_write_vertex<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError<R::Error>> {
        if B::INVERTED {
            self.try_write_vertex_inverted(writer)
        } else {
            self.try_write_vertex_normal(writer)
        }
    }

    /// Attempt to write the next vertex into the provided string buffer, failing to do so if the
    /// call to [`TryRamify::try_ramify`] results in an error.
    ///
    /// This is identical to [`try_write_vertex`](Self::try_write_vertex), except there is no
    /// IO error since writing will not fail (unless you run out of memory).
    pub fn try_write_vertex_str(&mut self, buf: &mut String) -> Result<bool, R::Error> {
        self.try_write_vertex(unsafe { buf.as_mut_vec() })
            .map_err(|e| match e {
                WriteVertexError::IO(_) => panic!("Out of memory!"),
                WriteVertexError::TryChildrenFailed(e) => e,
            })
    }

    fn try_write_vertex_normal<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError<R::Error>> {
        let mut writer = DiagramWriter::new(writer);

        // perform the substitution first since we will use information
        // about the next minimal element in order to make predictive writes
        let Some((col, marker_char)) = self
            .columns
            .try_substitute(&mut self.annotation_buf)
            .map_err(WriteVertexError::TryChildrenFailed)?
        else {
            return Ok(false);
        };

        // write the vertex row and get the diagram width
        let mut state = write_vertex_row(&mut self.columns, &mut writer, col, marker_char)?;

        let mut lines = self.annotation_buf.lines();

        // finish the vertex row and then write the annotation lines
        match lines.next() {
            Some(first_line) => {
                writer.write_annotation(first_line, &state)?;

                // write the remaining annotation lines
                for line in lines {
                    write_preparation_row(&mut self.columns, &mut writer, &mut state)?;
                    writer.write_annotation(line, &state)?;
                }
            }
            None => writer.write_newline()?,
        }

        // prepare for the next row, writing at least enough rows to get the desired
        // padding (except on the last row)
        if self.columns.is_empty() {
            Ok(false)
        } else {
            let mut padding = self.config().row_padding;
            while padding > 0 {
                write_preparation_row(&mut self.columns, &mut writer, &mut state)?;
                writer.write_newline()?;
                padding -= 1;
            }
            while !state.is_ready() {
                write_preparation_row(&mut self.columns, &mut writer, &mut state)?;
                writer.write_newline()?;
            }
            Ok(true)
        }
    }

    /// Try to write the next vertex in 'inverted' mode.
    ///
    /// Instead of writing the vertex and then preparing for the next vertex to be written, we
    /// start by preparing for the vertex row to be written and then write it last. We also write
    /// the padding that follows the row if we can determine that there will be another row.
    fn try_write_vertex_inverted<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError<R::Error>> {
        // substitute and update minimal index
        let Some((mut col, marker_char)) = self
            .columns
            .try_substitute(&mut self.annotation_buf)
            .map_err(WriteVertexError::TryChildrenFailed)?
        else {
            return Ok(false);
        };

        let mut writer = DiagramWriter::<W, B>::new(writer);

        // Write annotation lines, with the vertex on the last line.
        let mut lines = self.annotation_buf.lines();
        let maybe_last_line = lines.next();

        let mut state = match maybe_last_line {
            None => {
                // no annotation, so we can already prepare for the next row
                let state = write_vertex_row(&mut self.columns, &mut writer, col, marker_char)?;

                writer.write_newline()?;
                state
            }
            Some(last_line) => {
                match lines.next_back() {
                    None => {
                        let state = self.columns.write_shimmed_row(
                            &mut writer,
                            ops::Fork,
                            (col, ops::Marker(marker_char)),
                        )?;
                        writer.write_annotation(last_line, &state)?;

                        state
                    }
                    Some(first_line) => {
                        let mut state = write_preparation_row_inverted(
                            &mut self.columns,
                            &mut writer,
                            self.first,
                            &mut col,
                        )?;
                        writer.write_annotation(first_line, &state)?;

                        for line in lines.rev() {
                            let new = write_preparation_row_inverted(
                                &mut self.columns,
                                &mut writer,
                                self.first,
                                &mut col,
                            )?;
                            state.update(&new);
                            writer.write_annotation(line, &state)?;
                        }

                        let new_state = self.columns.write_shimmed_row(
                            &mut writer,
                            ops::Fork,
                            (col, ops::Marker(marker_char)),
                        )?;

                        // temporarily store the width, etc. in the previous state, and use it
                        // to write the annotation
                        state.update(&new_state);
                        writer.write_annotation(last_line, &state)?;

                        new_state
                    }
                }
            }
        };
        self.first = false;

        // write the padding and prepare for the next vertex (unless this is the last row)
        if self.columns.is_empty() {
            Ok(false)
        } else {
            let mut padding = self.config().row_padding;
            while padding > 0 {
                write_preparation_row(&mut self.columns, &mut writer, &mut state)?;
                writer.write_newline()?;
                padding -= 1;
            }

            // make the minimal index a singleton so that the vertex row can be written.
            while !state.is_ready() {
                write_preparation_row(&mut self.columns, &mut writer, &mut state)?;
                writer.write_newline()?;
            }
            Ok(true)
        }
    }
}

impl<V, R, B, P> Generator<V, R, B, P> {
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
    /// Note that multiple vertices may use the same edge. In particular, this number is
    /// distinct from the number of outgoing edges.
    ///
    /// Also note that there might be internal whitespace. In particular, this number is distinct
    /// from the actual width (in characters) of the diagram, even after taking into account the
    /// gutter width.
    pub fn girth(&self) -> usize {
        self.columns.girth()
    }

    /// Whether or not there are any active vertices.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// An iterator over the active vertices.
    pub fn into_active_vertices(self) -> impl ExactSizeIterator<Item = V> {
        self.columns.into_active_vertices()
    }

    /// Shrink the capacity of internal allocations as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.annotation_buf.shrink_to_fit();
        self.columns.shrink_to_fit();
    }
}

impl<V, R, P> Generator<V, R, RoundedCorners, P> {
    /// Initialize using default configuration with the *rounded corners* style.
    ///
    /// See the documentation for [`RoundedCorners`] for an example.
    pub fn with_rounded_corners(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_rounded_corners())
    }
}

impl<V, R, P> Generator<V, R, RoundedCornersWide, P> {
    /// Initialize using default configuration with the *rounded corners* style, and extra internal
    /// whitespace.
    ///
    /// See the documentation for [`RoundedCornersWide`] for an example.
    pub fn with_rounded_corners_wide(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_rounded_corners_wide())
    }
}

impl<V, R, P> Generator<V, R, SharpCorners, P> {
    /// Initialize using default configuration with the *sharp corners* style.
    ///
    /// See the documentation for [`SharpCorners`] for an example.
    pub fn with_sharp_corners(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_sharp_corners())
    }
}

impl<V, R, P> Generator<V, R, SharpCornersWide, P> {
    /// Initialize using default configuration with the *sharp corners* style, and extra internal
    /// whitespace.
    ///
    /// See the documentation for [`SharpCornersWide`] for an example.
    pub fn with_sharp_corners_wide(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_sharp_corners_wide())
    }
}

impl<V, R, P> Generator<V, R, DoubledLines, P> {
    /// Initialize using default configuration with the *doubled lines* style.
    ///
    /// See the documentation for [`DoubledLines`] for an example.
    pub fn with_doubled_lines(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_doubled_lines())
    }
}
