mod ops;
#[cfg(test)]
mod tests;

use std::{io, iter::repeat, ops::Range};

use crate::{
    Config, Ramify, TryRamify,
    writer::{
        DiagramWriter, DoubledLines, RoundedCorners, RoundedCornersWide, SharpCorners,
        SharpCornersWide, WriteBranch,
    },
};

/// A generator which incrementally writes the branch diagram to a writer.
///
/// Once you have a [`Ramify`] impementation, initialize this struct with the [`init`](Self::init) method. After initializing, the branch
/// diagram can be incrementally written to a [writer](io::Write) using the
/// [`write_next_vertex`](Self::write_next_vertex) method.
///
/// The documentation here is mostly relevant for using the [`Generator`]. The layout algorithm
/// is documented in the [`writer` module](crate::writer#layout-algorithm-documentation).
///
/// ## Compile-time and dynamic configuration
/// This struct can be configured by passing an appropriate [`Config`] struct. The configuration
/// contains compile-time and runtime configuration. The compile-time configuration is included in
/// the state parameter (for example, a [`RoundedCorners`] parameter), which describes the appearance of the
/// branch diagram. The runtime configuration concerns features relevant to the layout algorithm.
///
/// It is possible to modify configuration while writing the diagram (that is, in between calls to
/// [`write_next_vertex`](Self::write_next_vertex)) by using the [`config_mut`](Self::config_mut)
/// method. Any such modifications of the configuration are guaranteed to not
/// corrupt the branch diagram.
///
/// ## Runtime and memory complexity
///
/// The branch diagram generator holds the minimal possible state required to generate the diagram.
/// This state is essentially list of column indices corresponding to the *active vertices*: the vertices not yet drawn to the diagram, but
/// for which a parent has already been drawn to the diagram.
/// More precisely, the memory usage is `(4 + size_of<V>) * num_active_vertices` plus a constant
/// if you do not write annotations.
///
/// Writing a branch diagram row only requires making a single pass over the list of vertices.
/// Therefore the runtime to write a single branch diagram row is `O(num_active_vertices)`,
/// assuming the various methods in [`Ramify`] take constant time.
///
/// If an annotation is written, the entire annotation is loaded into a scratch buffer. The scratch
/// buffer is re-used between calls to [`write_next_vertex`](Self::write_next_vertex).
#[derive(Debug)]
pub struct Generator<V, R, B = RoundedCorners> {
    columns: Vec<(V, usize)>,
    min_index: Option<usize>, // None iff columns.is_empty()
    ramifier: R,
    config: Config<B>,
    annotation_buf: String,
}

impl<V, R, B: WriteBranch> Generator<V, R, B> {
    /// Get a new branch diagram generator starting at a given vertex of type `V` using the provided
    /// configuration.
    pub fn init(root: V, ramifier: R, config: Config<B>) -> Self {
        Self {
            columns: vec![(root, 0)],
            min_index: Some(0),
            ramifier,
            config,
            annotation_buf: String::new(),
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

    /// Get a mutable reference to the configuration in order to change configuration parameters
    /// while generating the branch diagram.
    pub fn config_mut(&mut self) -> &mut Config<B> {
        &mut self.config
    }
}

/// An error which might occur when calling [`Generator::try_write_next_vertex`].
#[derive(Debug)]
pub enum WriteVertexError {
    /// An IO error was raised by the writer.
    IO(io::Error),
    /// The [`TryRamify`] implementation failed to determine the children for the active vertex.
    TryChildrenFailed,
}

impl From<io::Error> for WriteVertexError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

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
    ///
    /// # Buffered writes
    ///
    /// The implementation tries to minimize the number of [`write`](io::Write::write) made by this method,
    /// but the number of calls is still large. It is recommended that the provided writer is
    /// buffered, for example using an [`io::BufWriter`] or an [`io::LineWriter`]. Many writers
    /// provided by the standard library are already buffered.
    pub fn write_next_vertex<W: io::Write>(&mut self, writer: W) -> io::Result<bool>
    where
        R: Ramify<V>,
    {
        self.try_write_next_vertex(writer).map_err(|e| match e {
            WriteVertexError::IO(error) => error,
            // the implementation of TryRamify if `R` is `Ramify` always succeeds
            WriteVertexError::TryChildrenFailed => unreachable!(),
        })
    }

    /// Attempt to write the next vertex, failing to do so if the call to [`TryRamify::try_children`]
    /// results in an error.
    ///
    /// If the call fails and the replacement vertex is different, this could still result in some
    /// rows written in order to prepare the new index to be written immediately if the next call
    /// succeeds. If the original vertex is returned on error it is guaranteed that no writes will be made.
    pub fn try_write_next_vertex<W: io::Write>(
        &mut self,
        writer: W,
    ) -> Result<bool, WriteVertexError>
    where
        R: TryRamify<V>,
    {
        let mut writer = DiagramWriter::<W, B>::new(writer);
        let Some(min_idx) = self.min_index else {
            return Ok(false);
        };

        // perform the substitution first since we will use information
        // about the next minimal element in order to make predictive writes
        let (marker_char, col, l, r) = {
            #[cfg(test)]
            self.debug_cols_header("Replacing min index");
            let original_col_count = self.columns.len();

            // use the 'sentinel' pattern
            let (marker_char, col) = if min_idx + 1 == self.columns.len() {
                // the minimal index is at the end

                // remove the last element
                let (vtx, col) = self.columns.pop().unwrap();

                // determine the data associated with the element
                let (marker_char, maybe_children) =
                    Self::get_vtx_data(&mut self.ramifier, &mut self.annotation_buf, vtx);

                // FIXME: annoying workaround to deal with borrow checker
                let children = if maybe_children.is_err() {
                    let replacement = unsafe { maybe_children.unwrap_err_unchecked() };
                    // put the column back, but with the replacement element
                    self.columns.push((replacement, col));

                    return Err(self.handle_no_children(&mut writer));
                } else {
                    unsafe { maybe_children.unwrap_unchecked() }
                };

                // append the new elements
                self.columns.extend(children.into_iter().zip(repeat(col)));

                (marker_char, col)
            } else {
                // temporarily swap the minimal element with the last element
                let (vtx, col) = self.columns.swap_remove(min_idx);

                // determine the data associated with the element
                let (marker_char, maybe_children) =
                    Self::get_vtx_data(&mut self.ramifier, &mut self.annotation_buf, vtx);

                // FIXME: annoying workaround to deal with borrow checker
                let children = if maybe_children.is_err() {
                    let replacement = unsafe { maybe_children.unwrap_err_unchecked() };
                    // put the column back with the replacement element
                    let last_idx = self.columns.len();
                    self.columns.push((replacement, col));
                    self.columns.swap(last_idx, min_idx);

                    return Err(self.handle_no_children(&mut writer));
                } else {
                    unsafe { maybe_children.unwrap_unchecked() }
                };

                // splice onto the swapped last element, inserting the new children
                let last = {
                    let mut iter = self
                        .columns
                        .splice(min_idx..min_idx + 1, children.into_iter().zip(repeat(col)));
                    iter.next().unwrap()
                };
                // put the last element back
                self.columns.push(last);

                (marker_char, col)
            };

            // update the min index
            self.min_index = self
                .columns
                .iter()
                .enumerate()
                .min_by_key(|(_, (e, _))| self.ramifier.get_key(e))
                .map(|(a, _)| a);

            // compute the number of new elements added by checking how much the length changed.
            let child_count = self.columns.len() + 1 - original_col_count;

            #[cfg(test)]
            self.debug_cols();

            (marker_char, col, min_idx, min_idx + child_count)
        };

        // either get the next minimal index, or write the final line and annotation and return
        let Some(next_min_idx) = self.min_index else {
            let (_, offset) = ops::marker(&mut writer, marker_char, 0, col)?;
            let diagram_width = self.compute_diagram_width(offset);
            let annotation_alignment = (B::GUTTER_WIDTH + 1) * diagram_width - B::GUTTER_WIDTH;

            let mut lines = self.annotation_buf.lines();

            if let Some(line) = lines.next() {
                writer.write_annotation_line(
                    line,
                    annotation_alignment,
                    self.config.annotation_margin,
                )?;
                for line in lines {
                    writer.write_annotation_line(
                        line,
                        annotation_alignment,
                        self.config.annotation_margin,
                    )?;
                }
            } else {
                writer.write_newline()?;
            }

            return Ok(false);
        };

        let diagram_width = self.compute_diagram_width_no_base(next_min_idx);

        let delay_fork = self.config.row_padding > 0;

        if next_min_idx < l {
            // the next minimal index lands before the marker

            let mut offset = if delay_fork {
                ops::fork_align::<_, _, _, false>(
                    &mut writer,
                    &mut self.columns[..l],
                    next_min_idx,
                    ..col,
                )?
            } else {
                ops::fork_align::<_, _, _, true>(
                    &mut writer,
                    &mut self.columns[..l],
                    next_min_idx,
                    ..col,
                )?
            };

            let (actual, next_offset) = ops::marker(&mut writer, marker_char, offset, col)?;
            offset = next_offset;
            if r < self.columns.len() {
                writer.queue_blank(offset.min(self.columns[r].1) - actual);
                ops::align(&mut writer, &mut self.columns[r..], offset..diagram_width)?;
            }
        } else if next_min_idx < r {
            // the next minimal index is a child of the marker

            // first, we use `align` on the preceding columns to make as much space as
            // possible. we can use the unbounded version since `align` by default compacts and this
            // may result in better codegen
            let mut offset = ops::align(&mut writer, &mut self.columns[..l], ..)?;
            let (actual, next_offset) = ops::mark_and_prepare(
                &mut writer,
                &self.columns,
                marker_char,
                offset,
                next_min_idx,
            )?;
            offset = next_offset;
            if r < self.columns.len() {
                writer.queue_blank(offset.min(self.columns[r].1) - actual);
                ops::align(&mut writer, &mut self.columns[r..], offset..diagram_width)?;
            }
        } else {
            // the next minimal index follows the marker

            let mut offset = ops::align(&mut writer, &mut self.columns[..l], ..)?;
            let (actual, next_offset) = ops::marker(&mut writer, marker_char, offset, col)?;
            offset = next_offset;
            if r < self.columns.len() {
                writer.queue_blank(offset.min(self.columns[r].1) - actual);
                if delay_fork {
                    ops::fork_align::<_, _, _, false>(
                        &mut writer,
                        &mut self.columns[r..],
                        next_min_idx - r,
                        offset..diagram_width,
                    )?;
                } else {
                    ops::fork_align::<_, _, _, true>(
                        &mut writer,
                        &mut self.columns[r..],
                        next_min_idx - r,
                        offset..diagram_width,
                    )?;
                }
            }
        };

        let annotation_alignment =
            ((B::GUTTER_WIDTH + 1) * diagram_width - B::GUTTER_WIDTH).max(writer.line_char_count());

        let mut lines = self.annotation_buf.lines();

        // write the first annotation line or a newline
        if let Some(line) = lines.next() {
            writer.write_annotation_line(
                line,
                annotation_alignment,
                self.config.annotation_margin,
            )?;
        } else {
            writer.write_newline()?;
        }

        #[cfg(test)]
        self.debug_cols_header(format!("Wrote marker line with marker {marker_char}"));

        // we prepare space for the next annotation, but don't fork until necessary
        if let Some(mut prev_line) = lines.next() {
            for line in lines {
                ops::fork_align::<_, _, _, false>(
                    &mut writer,
                    &mut self.columns,
                    next_min_idx,
                    ..diagram_width,
                )?;
                writer.write_annotation_line(
                    prev_line,
                    annotation_alignment,
                    self.config.annotation_margin,
                )?;
                #[cfg(test)]
                self.debug_cols_header("Wrote annotation line");

                prev_line = line;
            }

            ops::fork_align::<_, _, _, true>(
                &mut writer,
                &mut self.columns,
                next_min_idx,
                ..diagram_width,
            )?;
            writer.write_annotation_line(
                prev_line,
                annotation_alignment,
                self.config.annotation_margin,
            )?;
            #[cfg(test)]
            self.debug_cols_header("Wrote final annotation line");
        }

        // write some padding lines, and also prepare for the next row simultaneously
        for _ in 0..self.config.row_padding {
            self.try_make_singleton(next_min_idx, &mut writer, diagram_width)?;
        }

        // finally, prepare for the next row by repeatedly calling
        // `fork_align` until the index is a singleton, writing enough rows
        // to get the desired padding
        self.make_singleton(next_min_idx, &mut writer, diagram_width)?;

        Ok(true)
    }

    fn compute_min_idx(&self) -> Option<usize>
    where
        R: TryRamify<V>,
    {
        self.columns
            .iter()
            .enumerate()
            .min_by_key(|(_, (e, _))| self.ramifier.get_key(e))
            .map(|(a, _)| a)
    }

    fn get_vtx_data(
        ramifier: &mut R,
        buf: &mut String,
        vtx: V,
    ) -> (char, Result<impl IntoIterator<Item = V>, V>)
    where
        R: TryRamify<V>,
    {
        let marker_char = ramifier.marker(&vtx);
        buf.clear();
        ramifier
            .annotation(&vtx, buf)
            .expect("Writing to a `String` should not fail.");
        (marker_char, ramifier.try_children(vtx))
    }

    /// Write a row which tries to prepare for the next vertex.
    fn try_make_singleton<W: io::Write>(
        &mut self,
        idx: usize,
        writer: &mut DiagramWriter<W, B>,
        diagram_width: usize,
    ) -> io::Result<()> {
        ops::fork_align::<_, _, _, true>(writer, &mut self.columns, idx, ..diagram_width)?;
        writer.write_newline()?;
        #[cfg(test)]
        self.debug_cols_header("Wrote non-marker line");
        Ok(())
    }

    /// Given an index, repeatedly call `ops::fork_align` until the corresponding index is a singleton
    /// column.
    fn make_singleton<W: io::Write>(
        &mut self,
        idx: usize,
        writer: &mut DiagramWriter<W, B>,
        diagram_width: usize,
    ) -> io::Result<()> {
        while !self.is_singleton(idx) {
            self.try_make_singleton(idx, writer, diagram_width)?;
        }
        Ok(())
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
        self.columns.last().map(|(_, c)| *c)
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
        self.columns.len()
    }

    /// Whether or not there are any active vertices.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns whether a provided index is a singleton, i.e. the corresponding edge is not shared
    /// by any other vertices.
    fn is_singleton(&self, idx: usize) -> bool {
        let Range { start: l, end: r } = ops::column_range(&self.columns, idx);
        l + 1 == r
    }

    fn handle_no_children<W: io::Write>(
        &mut self,
        writer: &mut DiagramWriter<W, B>,
    ) -> WriteVertexError
    where
        R: TryRamify<V>,
    {
        // recompute the min index
        let new_min_idx = self.compute_min_idx().unwrap();

        self.min_index = Some(new_min_idx);

        // prepare to write the vertex next iteration
        let diagram_width = self.compute_diagram_width_no_base(new_min_idx);
        if let Err(e) = self.make_singleton(new_min_idx, writer, diagram_width) {
            return e.into();
        }
        WriteVertexError::TryChildrenFailed
    }

    /// Returns the amount of diagram width from the base diagram width (the amount of space
    /// required for all of the rows before writing the next vertex) and taking into account the
    /// configuration.
    fn compute_diagram_width(&self, base_diagram_width: usize) -> usize {
        let slack: usize = self.config.width_slack.into();
        (base_diagram_width + slack).max(self.config.min_diagram_width)
    }

    fn compute_diagram_width_no_base(&self, min_idx: usize) -> usize {
        self.compute_diagram_width(ops::required_width(&self.columns, min_idx))
    }

    #[cfg(test)]
    fn debug_cols(&self) {
        self.debug_cols_impl(None::<std::convert::Infallible>);
    }

    #[cfg(test)]
    fn debug_cols_header<D: std::fmt::Display>(&self, header: D) {
        self.debug_cols_impl(Some(header));
    }

    #[cfg(test)]
    fn debug_cols_impl<D: std::fmt::Display>(&self, header: Option<D>) {
        if let Some(min_idx) = self.min_index {
            if let Some(s) = header {
                println!("{s}:");
            }
            print!(" ->");
            for (i, (_, col)) in self.columns.iter().enumerate() {
                if i == min_idx {
                    print!(" *{col}");
                } else {
                    print!("  {col}");
                }
            }
            println!();
        } else {
            println!("Tree is empty");
        }
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
