mod columns;
mod ops;
#[cfg(test)]
mod tests;

use std::io;

use crate::{
    Config, Ramify, TryRamify,
    writer::{
        Branch, DiagramWriter, DoubledLines, RoundedCorners, RoundedCornersWide, SharpCorners,
        SharpCornersWide, WriteBranch,
    },
};

use self::columns::Columns;

/// A generator which incrementally writes the branch diagram to a writer.
///
/// Once you have a [`Ramify`] impementation, initialize this struct with the [`init`](Self::init) method. After initializing, the branch
/// diagram can be incrementally written to a [writer](io::Write) using the
/// [`write_vertex`](Self::write_vertex) method. You can also use the
/// [`branch_diagram`](Self::branch_diagram) method as a convenience function to load the entire
/// tree into memory.
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
/// - [`Ramify::annotation`] is called exactly once called to determine the annotation for the
///   minimal vertex.
/// - [`Ramify::children`] is called exactly once to replace the current minimal vertex with its
///   children
/// - [`Ramify::get_key`] is called once for every active vertex every time a new vertex is
///   generated.
///
/// Moreover, the call to [`Ramify::children`] is **guaranteed to be last** for each vertex. This is enforced by the borrow checker since the signature takes ownership of `V`.
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
/// been drawn to the diagram, it is passed to [`Ramify::children`] or [`TryRamify::try_children`],
/// which takes ownership of `V`.
///
/// A generator will *never* drop a vertex while it is running. Therefore all of the resource
/// management takes place in the [`Ramify`] or [`TryRamify`] implementation. If you drop a
/// generator, the list of active vertices will be de-allocated. You can recover the list of active vertices using [`into_active_vertices`](Self::into_active_vertices).
///
/// ### Runtime and memory complexity
///
/// The branch diagram generator holds the minimal possible state required to generate the diagram.
/// This state is essentially the active vertices plus additional metadata concerning the column to which the vertex belongs in the diagram.
/// More precisely, the memory usage is `(4 + size_of<V>) * num_active_vertices` plus a constant
/// if you do not write annotations.
///
/// Writing a branch diagram row only requires making a single pass over the list of vertices.
/// Therefore the runtime to write a single branch diagram row is `O(num_active_vertices)`,
/// assuming the various methods in [`Ramify`] take constant time.
///
/// If an annotation is written, the entire annotation is loaded into a scratch buffer. The scratch
/// buffer is re-used between calls to [`write_vertex`](Self::write_vertex).
#[derive(Debug)]
pub struct Generator<V, R, B = RoundedCorners> {
    columns: Columns<V, R, B>,
    min_index: Option<usize>, // None iff columns.is_empty()
    annotation_buf: String,
    // a special bit of state needed to handle 'inverted' mode correctly
    first: bool,
}

impl<V, R, B: WriteBranch> Generator<V, R, B> {
    /// Get a new branch diagram generator starting at a given vertex of type `V` using the provided
    /// configuration.
    pub fn init(root: V, ramifier: R, config: Config<B>) -> Self {
        Self {
            columns: Columns::init(root, ramifier, config),
            min_index: Some(0),
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
    pub fn config(&mut self) -> &Config<B> {
        self.columns.config_mut()
    }

    /// Returns a mutable reference to the configuration.
    ///
    /// The configuration parameters can be safely changed while generating the branch diagram.
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

impl<E: std::error::Error> std::error::Error for WriteVertexError<E> {}

impl<V, R, B: WriteBranch> Generator<V, R, B> {
    /// Write a row containing a vertex along with its annotation to the provided writer.
    ///
    /// This method returns `Ok(true)` if there are vertices remaining, and otherwise `Ok(false)`.
    ///
    /// # Output rows
    ///
    /// A single call to this method will first write the row containing the vertex. Then, it will
    /// write a number of non-marker rows in order to accommodate additional lines of annotation
    /// and to set the generator state so that the subsequent call can immediately write
    /// a vertex.
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
    ///
    /// # Inverted mode
    ///
    /// In inverted mode, the vertex is written last rather than first, and
    /// the annotation lines are written in reverse order. This makes the annotations look correct
    /// if the tree is displayed with the root at the bottom.
    pub fn write_vertex<W: io::Write>(&mut self, writer: W) -> io::Result<bool>
    where
        R: Ramify<V>,
    {
        self.try_write_vertex(writer).map_err(|e| match e {
            WriteVertexError::IO(error) => error,
            // the implementation of TryRamify if `R` is `Ramify` always succeeds
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

    /// Attempt to write the next vertex, failing to do so if the call to [`TryRamify::try_children`]
    /// results in an error.
    ///
    /// The error is propagated in the [`WriteVertexError`] and can be used
    /// by the caller to decide whether or not to attempt to write the row again.
    ///
    /// # Handling of the replacement vertex
    ///
    /// If the call fails and the replacement vertex is different, this could still result in some
    /// rows written in order to prepare the new vertex to be written immediately if the next call
    /// succeeds.
    ///
    /// If the replacement vertex is still the minimal vertex, it is guaranteed that no writes will
    /// occur. This is the case if the original vertex is returned and [`TryRamify::get_key`] is a pure function.
    pub fn try_write_vertex<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError<R::Error>>
    where
        R: TryRamify<V>,
    {
        if B::INVERTED {
            self.try_write_vertex_inverted(writer)
        } else {
            self.first = false;
            self.try_write_vertex_normal(writer)
        }
    }

    /// Attempt to write the next vertex into the provided string buffer, failing to do so if the
    /// call to [`TryRamify::try_children`] results in an error.
    ///
    /// This is identical to [`try_write_vertex`](Self::try_write_vertex), except there is no
    /// IO error since writing will not fail (unless you run out of memory).
    pub fn try_write_vertex_str(&mut self, buf: &mut String) -> Result<bool, R::Error>
    where
        R: TryRamify<V>,
    {
        self.try_write_vertex(unsafe { buf.as_mut_vec() })
            .map_err(|e| match e {
                WriteVertexError::IO(_) => panic!("Out of memory!"),
                WriteVertexError::TryChildrenFailed(e) => e,
            })
    }

    fn try_write_vertex_normal<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError<R::Error>>
    where
        R: TryRamify<V>,
    {
        let Some(min_idx) = self.min_index else {
            return Ok(false);
        };

        let mut writer = DiagramWriter::<W, B>::new(writer);

        // perform the substitution first since we will use information
        // about the next minimal element in order to make predictive writes
        let marker_char = self.columns.marker_char(min_idx);
        let col = self.columns.col(min_idx);
        self.annotation_buf.clear();
        self.columns
            .buffer_annotation(min_idx, &mut self.annotation_buf);
        let (l, r) =
            Self::sub_and_update_min(&mut self.columns, &mut self.min_index, min_idx, &mut writer)?;

        // either get the next minimal index, or write the final line and annotation and return
        let Some(next_min_idx) = self.min_index else {
            let (_, offset) = ops::marker(&mut writer, marker_char, 0, col)?;
            let diagram_width = self.config().normalize_diagram_width(offset);
            let annotation_alignment = (B::GUTTER_WIDTH + 1) * diagram_width - B::GUTTER_WIDTH;

            // write the annotation lines
            if self.annotation_buf.is_empty() {
                writer.write_newline()?;
            } else {
                for line in self.annotation_buf.lines() {
                    self.columns
                        .write_annotation_line(&mut writer, line, annotation_alignment)?;
                }
            }

            // don't write padding

            return Ok(false);
        };

        // data used to render the remaining rows
        let diagram_width = self.columns.diagram_width(next_min_idx);
        let delay_fork = self.config().row_padding > 0;

        // write the vertex row
        self.columns.write_vertex_row(
            &mut writer,
            next_min_idx,
            l,
            r,
            delay_fork,
            col,
            marker_char,
            diagram_width,
        )?;

        // compute the annotation alignment based on how large the vertex row is
        let annotation_alignment =
            ((B::GUTTER_WIDTH + 1) * diagram_width - B::GUTTER_WIDTH).max(writer.line_char_count());

        self.columns.write_trailing_annotation(
            self.annotation_buf.lines(),
            &mut writer,
            next_min_idx,
            diagram_width,
            annotation_alignment,
        )?;

        // finally, prepare for the next row by repeatedly calling
        // `fork_align` until the index is a singleton, writing at least
        // enough rows to get the desired padding
        let padding = self.config().row_padding;
        self.columns
            .make_singleton(next_min_idx, &mut writer, diagram_width, padding)?;

        Ok(true)
    }

    /// Try to write the next vertex in 'inverted' mode.
    ///
    /// Instead of writing the vertex and then preparing for the next vertex to be written, we
    /// start by preparing for the vertex row to be written and then write it last. We also write
    /// the padding that follows the row if we can determine that there will be another row.
    fn try_write_vertex_inverted<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError<R::Error>>
    where
        R: TryRamify<V>,
    {
        let Some(min_idx) = self.min_index else {
            return Ok(false);
        };

        let diagram_width = self.columns.diagram_width(min_idx);
        let mut writer = DiagramWriter::<W, B>::new(writer);

        // hard-code the root vertex to not print lines underneath it
        if self.first {
            self.first = false;

            // get all of the data for the minimal vertex
            let marker_char = self.columns.marker_char(min_idx);
            self.annotation_buf.clear();
            self.columns
                .buffer_annotation(min_idx, &mut self.annotation_buf);

            // write the annotation lines and the marker
            let mut lines = self.annotation_buf.lines();
            let maybe_last_line = lines.next();
            match maybe_last_line {
                Some(last_line) => {
                    for line in lines.rev() {
                        self.columns.write_annotation_line(&mut writer, line, 1)?;
                    }
                    ops::marker(&mut writer, marker_char, 0, 0)?;
                    self.columns
                        .write_annotation_line(&mut writer, last_line, 1)?;
                }
                None => {
                    ops::marker(&mut writer, marker_char, 0, 0)?;
                    writer.write_newline()?;
                }
            }

            // substitute the vertex and return
            Self::sub_and_update_min(&mut self.columns, &mut self.min_index, min_idx, &mut writer)?;
            return Ok(self.min_index.is_some());
        }

        // write the padding, and also preparing for next vertex with updated diagram width
        for _ in 0..self.config().row_padding {
            self.columns
                .try_make_singleton(min_idx, &mut writer, diagram_width)?;
        }

        // make the minimal index a singleton so that the vertex row can be written.
        // TODO: can we predict how long it will take to make a singleton? probably hard
        self.columns
            .make_singleton(min_idx, &mut writer, diagram_width, 0)?;

        // get all of the data for the minimal vertex
        let marker_char = self.columns.marker_char(min_idx);
        self.annotation_buf.clear();
        self.columns
            .buffer_annotation(min_idx, &mut self.annotation_buf);

        // write annotation lines, with the vertex on the last line
        let mut lines = self.annotation_buf.lines();
        let maybe_last_line = lines.next();

        match maybe_last_line {
            None => {
                // no annotation

                // substitute and update minimal index
                let col = self.columns.col(min_idx);
                let (l, r) = Self::sub_and_update_min(
                    &mut self.columns,
                    &mut self.min_index,
                    min_idx,
                    &mut writer,
                )?;

                match self.min_index {
                    Some(next_min_idx) => {
                        // the next min index exists, so we write the vertex row and prepare for
                        // the next write
                        self.columns.write_vertex_row(
                            &mut writer,
                            next_min_idx,
                            l,
                            r,
                            false,
                            col,
                            marker_char,
                            diagram_width,
                        )?;
                        writer.write_newline()?;
                        Ok(true)
                    }
                    None => {
                        // no more vertices, so we just write the marker row
                        writer.queue_blank(col);
                        writer.write_branch(Branch::Marker(marker_char))?;
                        writer.write_newline()?;
                        Ok(false)
                    }
                }
            }
            Some(last_line) => {
                // write all of the preceding annotation lines
                let maybe_alignment = self
                    .columns
                    .write_preceding_annotation(lines.rev(), &mut writer)?;

                // substitute and update minimal index
                let col = self.columns.col(min_idx);
                let (l, r) = Self::sub_and_update_min(
                    &mut self.columns,
                    &mut self.min_index,
                    min_idx,
                    &mut writer,
                )?;

                match self.min_index {
                    Some(next_min_idx) => {
                        // the next min index exists, so we write the vertex row and prepare for
                        // the next write
                        self.columns.write_vertex_row(
                            &mut writer,
                            next_min_idx,
                            l,
                            r,
                            false,
                            col,
                            marker_char,
                            diagram_width,
                        )?;
                        let alignment = maybe_alignment.unwrap_or(writer.line_char_count());
                        self.columns
                            .write_annotation_line(&mut writer, last_line, alignment)?;

                        Ok(true)
                    }
                    None => {
                        // no more vertices, so we just write the marker row
                        writer.queue_blank(col);
                        writer.write_branch(Branch::Marker(marker_char))?;
                        let alignment = maybe_alignment.unwrap_or(col + 1);
                        self.columns
                            .write_annotation_line(&mut writer, last_line, alignment)?;
                        Ok(false)
                    }
                }
            }
        }
    }

    fn sub_and_update_min<W: io::Write>(
        self_columns: &mut Columns<V, R, B>,
        self_min_index: &mut Option<usize>,
        min_idx: usize,
        writer: &mut DiagramWriter<W, B>,
    ) -> Result<(usize, usize), WriteVertexError<R::Error>>
    where
        R: TryRamify<V>,
    {
        // substitute and update minimal index
        match self_columns.substitute(min_idx) {
            Ok((l, r)) => {
                *self_min_index = self_columns.min_index();
                Ok((l, r))
            }
            Err(e) => {
                // recompute the min index
                let new_min_idx = self_columns.min_index().unwrap();

                *self_min_index = Some(new_min_idx);

                // prepare to write the vertex next iteration
                let diagram_width = self_columns.diagram_width(new_min_idx);
                self_columns.make_singleton(new_min_idx, writer, diagram_width, 0)?;
                Err(WriteVertexError::TryChildrenFailed(e))
            }
        }
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
}

impl<V, R> Generator<V, R, RoundedCorners> {
    /// Initialize using default configuration with the *rounded corners* style.
    ///
    /// See the documentation for [`RoundedCorners`] for an example.
    pub fn with_rounded_corners(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_rounded_corners())
    }
}

impl<V, R> Generator<V, R, RoundedCornersWide> {
    /// Initialize using default configuration with the *rounded corners* style, and extra internal
    /// whitespace.
    ///
    /// See the documentation for [`RoundedCornersWide`] for an example.
    pub fn with_rounded_corners_wide(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_rounded_corners_wide())
    }
}

impl<V, R> Generator<V, R, SharpCorners> {
    /// Initialize using default configuration with the *sharp corners* style.
    ///
    /// See the documentation for [`SharpCorners`] for an example.
    pub fn with_sharp_corners(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_sharp_corners())
    }
}

impl<V, R> Generator<V, R, SharpCornersWide> {
    /// Initialize using default configuration with the *sharp corners* style, and extra internal
    /// whitespace.
    ///
    /// See the documentation for [`SharpCornersWide`] for an example.
    pub fn with_sharp_corners_wide(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_sharp_corners_wide())
    }
}

impl<V, R> Generator<V, R, DoubledLines> {
    /// Initialize using default configuration with the *doubled lines* style.
    ///
    /// See the documentation for [`DoubledLines`] for an example.
    pub fn with_doubled_lines(root: V, ramifier: R) -> Self {
        Self::init(root, ramifier, Config::with_doubled_lines())
    }
}
