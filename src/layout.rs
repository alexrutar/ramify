mod ops;
#[cfg(test)]
mod tests;

use std::{io, iter::repeat, ops::Range};

use crate::{
    Config, Ramify,
    writer::{
        BranchWrite, DiagramWriter, DoubledLines, RoundedCorners, RoundedCornersWide, SharpCorners,
        SharpCornersWide,
    },
};

/// A generator holding the state of a branch diagram at a single point in time.
///
/// Initialize this struct with the [`init`](Self::init) method. After initializing, the branch
/// diagram can be incrementally written to a [writer](io::Write) using the
/// [`write_next_vertex`](Self::write_next_vertex) method.
///
/// ## Internal state
/// The generator corresponds to the state at the `tip` of a partially written branch diagram. In
/// order to reduce the width of the branch diagram, multiple vertices can share the same edges
/// within the diagram.
///
/// For example, consider the following partial branch diagram. The vertex `0` is the root.
///
/// We can see that it has children `3`, `1`, and `2`. The vertex `2` also has a child `4`. These
/// vertices also have an unknown number of children that have not yet been drawn, corresponding to the
/// outgoing edges at the bottom of the diagram.
/// ```txt
/// 0
/// ├┬╮
/// │1│
/// ├╮2
/// 3│├╮
/// │││4
/// ```
pub struct Generator<V, R, B = RoundedCorners> {
    columns: Vec<(V, usize)>,
    ramifier: R,
    config: Config<B>,
    annotation_buf: String,
}

impl<V, R, B: BranchWrite> Generator<V, R, B> {
    /// Get a new branch diagram generator starting at a given vertex of type `V` using the provided
    /// configuration.
    pub fn init(root: V, ramifier: R, config: Config<B>) -> Self {
        Self {
            columns: vec![(root, 0)],
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
        Self {
            columns: vec![(root, 0)],
            ramifier,
            config: Config::new(),
            annotation_buf: String::new(),
        }
    }

    /// Get a mutable reference to the configuration in order to change configuration parameters
    /// while generating branch diagram.
    pub fn config_mut(&mut self) -> &mut Config<B> {
        &mut self.config
    }
}

impl<V: Copy, R: Ramify<V>, B: BranchWrite> Generator<V, R, B> {
    /// Write a row containing a vertex along with its annotation to the provided writer.
    ///
    /// This method returns `Ok(true)` if there are vertices remaining, and otherwise `Ok(false)`.
    ///
    /// ## Output rows
    ///
    /// A single call to this method will first write the row containing the vertex. Then, it will
    /// write a number of non-marker rows in order to accommodate additional lines of annotation
    /// and to set the generator state so that the subsequent call can immediately write
    /// a vertex.
    ///
    ///
    /// ## Buffered writes
    ///
    /// The implementation tries to minimize the number of calls to `write!` made by this method,
    /// but the number of calls is still large. It is recommended that the provided writer is
    /// buffered, for example using an [`io::BufWriter`] or an [`io::LineWriter`]. Many writers
    /// provided by the standard library are already buffered.
    pub fn write_next_vertex<W: io::Write>(&mut self, writer: W) -> io::Result<bool> {
        let mut writer = DiagramWriter::<W, B>::new(writer);
        let Some(min_idx) = self.min_index() else {
            return Ok(false);
        };

        // perform the substitution first since we will use information
        // about the next minimal element in order to make predictive writes
        let (vtx, col, l, r) = {
            #[cfg(test)]
            self.debug_cols_header("Replacing min index");
            let original_col_count = self.columns.len();
            let (vtx, col) = self.columns[min_idx];

            // replace this element with its children in place
            self.columns.splice(
                min_idx..min_idx + 1,
                // also store the column index from which the item originated
                self.ramifier.children(vtx).zip(repeat(col)),
            );

            // compute the number of new elements added by checking how much the length changed.
            let child_count = self.columns.len() + 1 - original_col_count;

            #[cfg(test)]
            self.debug_cols();

            (vtx, col, min_idx, min_idx + child_count)
        };

        let marker_char = self.ramifier.marker(vtx);

        // either get the next minimal index, or write the final line and annotation and return
        let Some(next_min_idx) = self.min_index() else {
            let diagram_width = ops::marker(&mut writer, marker_char, 0, col)?;
            let annotation_alignment = if B::WIDE {
                2 * diagram_width - 1
            } else {
                diagram_width
            };

            self.annotation_buf.clear();
            self.ramifier
                .annotation(
                    vtx,
                    self.config.margin_left + annotation_alignment,
                    &mut self.annotation_buf,
                )
                .expect("Writing to a `String` should not fail.");

            let mut lines = self.annotation_buf.lines();

            if let Some(line) = lines.next() {
                writer.write_annotation_line(
                    line,
                    annotation_alignment,
                    self.config.margin_left,
                )?;
                for line in lines {
                    writer.write_annotation_line(
                        line,
                        annotation_alignment,
                        self.config.margin_left,
                    )?;
                }
            } else {
                writer.write_newline()?;
            }

            return Ok(false);
        };

        // TODO: work out other strategies
        //
        // Option 1: Also take maximum with current width? Are there cases where
        //           this is better / worse?
        // Option 2: Greedy: set this to current width + 2, so the fork can immediately
        //           get more space if needed
        // Option 3: Allow some slack parameter u >= 0, which we just add.
        //
        // Handling these cases causes more difficulty with annotations since we need
        // to predict how much of the slack space we will actually use
        let diagram_width = ops::required_width(&self.columns, next_min_idx);

        let delay_fork = self.config.margin_below > 0;

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

            offset = ops::marker(&mut writer, marker_char, offset, col)?;
            ops::align(&mut writer, &mut self.columns[r..], offset..diagram_width)?;
        } else if next_min_idx < r {
            // the next minimal index is a child of the marker

            // first, we use `align` on the preceding columns to make as much space as
            // possible. we can use the unbounded version since `align` by default compats and this
            // may result in better codegen
            let mut offset = ops::align(&mut writer, &mut self.columns[..l], ..)?;
            offset = ops::mark_and_prepare(
                &mut writer,
                &self.columns,
                marker_char,
                offset,
                next_min_idx,
            )?;
            ops::align(&mut writer, &mut self.columns[r..], offset..diagram_width)?;
        } else {
            // the next minimal index follows the marker

            let mut offset = ops::align(&mut writer, &mut self.columns[..l], ..)?;
            offset = ops::marker(&mut writer, marker_char, offset, col)?;
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
        };

        let annotation_alignment = if B::WIDE {
            (2 * diagram_width - 1).max(writer.line_char_count())
        } else {
            diagram_width.max(writer.line_char_count())
        };

        self.annotation_buf.clear();
        self.ramifier
            .annotation(
                vtx,
                self.config.margin_left + annotation_alignment,
                &mut self.annotation_buf,
            )
            .expect("Writing to a `String` should not fail.");

        let mut lines = self.annotation_buf.lines();

        // write the first annotation line or a newline
        if let Some(line) = lines.next() {
            writer.write_annotation_line(line, annotation_alignment, self.config.margin_left)?;
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
                    self.config.margin_left,
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
                self.config.margin_left,
            )?;
            #[cfg(test)]
            self.debug_cols_header("Wrote final annotation line");
        }

        // write some padding lines, and also prepare for the next row simultaneously
        for _ in 0..self.config.margin_below {
            ops::fork_align::<_, _, _, true>(
                &mut writer,
                &mut self.columns,
                next_min_idx,
                ..diagram_width,
            )?;
            writer.write_newline()?;
            #[cfg(test)]
            self.debug_cols_header("Wrote padding line");
        }

        // finally, prepare for the next row by repeatedly calling
        // `fork_align` until the index is a singleton, writing enough rows
        // to get the desired padding
        while !self.is_singleton(next_min_idx) {
            ops::fork_align::<_, _, _, true>(
                &mut writer,
                &mut self.columns,
                next_min_idx,
                ..diagram_width,
            )?;
            writer.write_newline()?;
            #[cfg(test)]
            self.debug_cols_header("Wrote non-marker line");
        }

        Ok(true)
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
    /// edge index prior to writing a row, and then adding `1`.
    pub fn max_edge_index(&self) -> Option<usize> {
        self.columns.last().map(|(_, c)| *c)
    }

    /// The number of active vertices.
    ///
    /// Note that multiple vertices may use the same edge. In particular, this number is
    /// distinct from the number of outgoing edges.
    pub fn girth(&self) -> usize {
        self.columns.len()
    }

    /// Whether or not there are any active vertices.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Compute the column index containing the minimal key, or `None` if there are no columns.
    fn min_index(&self) -> Option<usize> {
        self.columns
            .iter()
            .enumerate()
            .min_by_key(|(_, (e, _))| self.ramifier.get_key(*e))
            .map(|(a, _)| a)
    }

    /// Returns whether a provided index is a singleton, i.e. the corresponding edge is not shared
    /// by any other vertices.
    fn is_singleton(&self, idx: usize) -> bool {
        let Range { start: l, end: r } = ops::column_range(&self.columns, idx);
        l + 1 == r
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
        if self.columns.is_empty() {
            println!("Tree is empty");
        } else {
            if let Some(s) = header {
                println!("{s}:");
            }
            print!(" ->");
            for (v, col) in &self.columns {
                print!(" ({} {col})", self.ramifier.marker(*v));
            }
            println!();
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
