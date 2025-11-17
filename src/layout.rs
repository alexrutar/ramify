mod ops;
#[cfg(test)]
mod tests;

use std::{io, iter::repeat, ops::Range};

use crate::{Ramify, writer::DiagramWriter};

/// A branch diagram.
pub struct BranchDiagram<V, R, W> {
    columns: Vec<(V, usize)>,
    ramifier: R,
    writer: DiagramWriter<W>,
    annotation_buf: String,
}

impl<V: Copy, R: Ramify<V>, W: io::Write> BranchDiagram<V, R, W> {
    pub fn new(root: V, ramifier: R, writer: DiagramWriter<W>) -> Self {
        Self {
            columns: vec![(root, 0)],
            // min_index: Some(0),
            ramifier,
            writer,
            annotation_buf: String::new(),
        }
    }

    /// The index of the final `open` edge, or `None` if there are no edges.
    ///
    /// For example, the diagram has maximum edge
    /// ```txt
    /// 0
    /// ├┬╮
    /// │1│
    /// ├╮╰─╮
    /// ```
    /// has maximum edge index `4`.
    ///
    /// This is not the same as the width of the diagram row which was previously written. However,
    /// we can use this information to compute the width of the diagram row by taking the maximum of the edge index and the
    /// edge index prior to writing a row, and then adding `1`.
    pub fn max_edge_index(&self) -> Option<usize> {
        self.columns.last().map(|(_, c)| *c)
    }

    /// The girth of the diagram, i.e. the number of active vertices.
    ///
    /// Since vertices can share edges, this is different than the number of outgoing edges.
    pub fn girth(&self) -> usize {
        self.columns.len()
    }

    /// Whether or not there are any active vertices.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    fn min_index(&self) -> Option<usize> {
        self.columns
            .iter()
            .enumerate()
            .min_by_key(|(_, (e, _))| self.ramifier.get_key(*e))
            .map(|(a, _)| a)
    }

    fn replace_index(&mut self, idx: usize) -> (V, usize, Range<usize>) {
        #[cfg(test)]
        self.debug_cols_header("Replacing min index");
        let original_col_count = self.columns.len();
        let (vtx, col) = self.columns[idx];

        // replace this element with its children in place
        self.columns.splice(
            idx..idx + 1,
            // also store the column index from which the item originated
            self.ramifier.children(vtx).zip(repeat(col)),
        );

        // compute the number of new elements added by checking how much the length changed.
        let child_count = self.columns.len() + 1 - original_col_count;

        #[cfg(test)]
        self.debug_cols();

        (vtx, col, idx..idx + child_count)
    }

    fn is_singleton(&self, idx: usize) -> bool {
        let Range { start: l, end: r } = ops::column_range(&self.columns, idx);
        l + 1 == r
    }

    /// Write a line containing a marker assuming, along will all of the subsequent lines.
    pub fn write_next_marker(&mut self) -> io::Result<bool> {
        let Some(min_idx) = self.min_index() else {
            return Ok(false);
        };

        // perform the substitution first since we will use information
        // about the next minimal element in order to make predictive writes
        let (vtx, col, Range { start: l, end: r }) = self.replace_index(min_idx);
        let marker_char = self.ramifier.marker(vtx);

        // either get the next minimal index, or write the final line and annotation and return
        let Some(next_min_idx) = self.min_index() else {
            let diagram_width = ops::marker(&mut self.writer, marker_char, 0, col)?;

            self.annotation_buf.clear();
            self.ramifier
                .annotation(vtx, diagram_width, &mut self.annotation_buf);

            for line in self.annotation_buf.lines() {
                self.writer.write_annotation_line(line, diagram_width)?;
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

        self.annotation_buf.clear();
        self.ramifier
            .annotation(vtx, diagram_width, &mut self.annotation_buf);
        let mut lines = self.annotation_buf.lines();

        if next_min_idx < l {
            // the next minimal index lands before the marker

            // TODO: when we do multiline / margin, we cannot immediately fork here; instead, we
            // need this method and also a 'prepare fork' method, and to call the `fork_align`
            // method only if we know there are no more annotation lines / padding to be written

            // we use `..col` since want to prepare space to fork, but we cannot exceed the marker
            // position
            let mut offset = ops::fork_align::<_, _, true>(
                &mut self.writer,
                &mut self.columns[..l],
                next_min_idx,
                ..col,
            )?;

            offset = ops::marker(&mut self.writer, marker_char, offset, col)?;
            ops::align(
                &mut self.writer,
                &mut self.columns[r..],
                offset..diagram_width,
            )?;
        } else if next_min_idx < r {
            // the next minimal index is a child of the marker

            // first, we use `align` on the preceding columns to make as much space as
            // possible. we can use the unbounded version since `align` by default compats and this
            // may result in better codegen
            let mut offset = ops::align(&mut self.writer, &mut self.columns[..l], ..)?;
            offset = ops::mark_and_prepare(
                &mut self.writer,
                &self.columns,
                marker_char,
                offset,
                next_min_idx,
            )?;
            ops::align(
                &mut self.writer,
                &mut self.columns[r..],
                offset..diagram_width,
            )?;
        } else {
            // the next minimal index follows the marker

            let mut offset = ops::align(&mut self.writer, &mut self.columns[..l], ..)?;
            offset = ops::marker(&mut self.writer, marker_char, offset, col)?;
            ops::fork_align::<_, _, true>(
                &mut self.writer,
                &mut self.columns[r..],
                next_min_idx - r,
                offset..diagram_width,
            )?;
        };

        let annotation_alignment = diagram_width.max(self.writer.line_char_count());

        // write the first annotation line
        if let Some(line) = lines.next() {
            self.writer
                .write_annotation_line(line, annotation_alignment)?;
        }

        #[cfg(test)]
        self.debug_cols_header(format!("Wrote marker line with marker {marker_char}"));

        // we prepare space for the next annotation, but don't fork until necessary
        if let Some(mut prev_line) = lines.next() {
            for line in lines {
                ops::fork_align::<_, _, false>(
                    &mut self.writer,
                    &mut self.columns,
                    next_min_idx,
                    ..diagram_width,
                )?;
                self.writer
                    .write_annotation_line(prev_line, annotation_alignment)?;
                #[cfg(test)]
                self.debug_cols_header("Wrote annotation line");

                prev_line = line;
            }

            ops::fork_align::<_, _, true>(
                &mut self.writer,
                &mut self.columns,
                next_min_idx,
                ..diagram_width,
            )?;
            self.writer
                .write_annotation_line(prev_line, annotation_alignment)?;
            #[cfg(test)]
            self.debug_cols_header("Wrote final annotation line");
        }

        // write some padding lines, and also prepare for the next row simultaneously
        for _ in 0..self.writer.config.annotation_margin_below {
            ops::fork_align::<_, _, true>(
                &mut self.writer,
                &mut self.columns,
                next_min_idx,
                ..diagram_width,
            )?;
            self.writer.write_newline()?;
            #[cfg(test)]
            self.debug_cols_header("Wrote padding line");
        }

        // finally, prepare for the next row by repeatedly calling
        // `fork_align` until the index is a singleton, writing enough rows
        // to get the desired padding
        while !self.is_singleton(next_min_idx) {
            ops::fork_align::<_, _, true>(
                &mut self.writer,
                &mut self.columns,
                next_min_idx,
                ..diagram_width,
            )?;
            self.writer.write_newline()?;
            #[cfg(test)]
            self.debug_cols_header("Wrote non-marker line");
        }

        Ok(true)
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
            debug_assert!(self.columns.iter().is_sorted_by_key(|(_, c)| c));
        }
    }
}
