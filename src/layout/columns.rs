use super::ops;

use std::{io, iter::repeat, ops::Range};

use crate::{
    TryRamify,
    writer::Config,
    writer::{DiagramWriter, WriteBranch},
};

#[derive(Debug)]
pub struct Columns<V, R, B> {
    columns: Vec<(V, usize)>,
    ramifier: R,
    config: Config<B>,
}

impl<V, R, B> Columns<V, R, B> {
    pub fn config(&self) -> &Config<B> {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config<B> {
        &mut self.config
    }

    pub fn init(root: V, ramifier: R, config: Config<B>) -> Self {
        Self {
            columns: vec![(root, 0)],
            ramifier,
            config,
        }
    }

    pub fn max_edge_index(&self) -> Option<usize> {
        self.columns.last().map(|(_, c)| *c)
    }

    pub fn girth(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

impl<V, R: TryRamify<V>, B: WriteBranch> Columns<V, R, B> {
    pub fn marker_char(&self, idx: usize) -> char {
        self.ramifier.marker(&self.columns[idx].0)
    }

    pub fn col(&self, idx: usize) -> usize {
        self.columns[idx].1
    }

    /// Compute the annotation, storing it in the provided buffer.
    pub fn buffer_annotation(&mut self, idx: usize, buf: &mut String) {
        self.ramifier
            .annotation(&self.columns[idx].0, buf)
            .expect("Writing to a `String` should not fail.");
    }

    pub fn substitute(&mut self, min_idx: usize) -> Option<(usize, usize)> {
        let original_col_count = self.columns.len();

        // use the 'sentinel' pattern
        if min_idx + 1 == self.columns.len() {
            // the minimal index is at the end

            // remove the last element
            let (vtx, col) = self.columns.pop().unwrap();

            // determine the data associated with the element
            let maybe_children = self.ramifier.try_children(vtx);

            // FIXME: annoying workaround to deal with borrow checker
            let children = if maybe_children.is_err() {
                let replacement = unsafe { maybe_children.unwrap_err_unchecked() };
                // put the column back, but with the replacement element
                self.columns.push((replacement, col));

                return None;
            } else {
                unsafe { maybe_children.unwrap_unchecked() }
            };

            // append the new elements
            self.columns.extend(children.into_iter().zip(repeat(col)));
        } else {
            // temporarily swap the minimal element with the last element
            let (vtx, col) = self.columns.swap_remove(min_idx);

            let maybe_children = self.ramifier.try_children(vtx);

            // FIXME: annoying workaround to deal with borrow checker
            let children = if maybe_children.is_err() {
                let replacement = unsafe { maybe_children.unwrap_err_unchecked() };
                // put the column back with the replacement element
                let last_idx = self.columns.len();
                self.columns.push((replacement, col));
                self.columns.swap(last_idx, min_idx);

                return None;
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
        };

        // compute the number of new elements added by checking how much the length changed.
        let child_count = self.columns.len() + 1 - original_col_count;

        Some((min_idx, min_idx + child_count))
    }

    pub fn diagram_width(&self, min_idx: usize) -> usize {
        self.config
            .normalize_diagram_width(ops::required_width(&self.columns, min_idx))
    }

    /// Get the minimal index, if any.
    pub fn min_index(&self) -> Option<usize>
    where
        R: TryRamify<V>,
    {
        self.columns
            .iter()
            .enumerate()
            .min_by_key(|(_, (e, _))| self.ramifier.get_key(e))
            .map(|(a, _)| a)
    }

    /// Given an index, repeatedly call `ops::fork_align` until the corresponding index is a singleton
    /// column.
    pub fn make_singleton<W: io::Write>(
        &mut self,
        idx: usize,
        writer: &mut DiagramWriter<W, B>,
        diagram_width: usize,
        row_padding: usize,
    ) -> io::Result<()> {
        for _ in 0..row_padding {
            self.try_make_singleton(idx, writer, diagram_width)?;
        }
        while !self.is_singleton(idx) {
            self.try_make_singleton(idx, writer, diagram_width)?;
        }
        Ok(())
    }

    /// Returns whether a provided index is a singleton, i.e. the corresponding edge is not shared
    /// by any other vertices.
    pub fn is_singleton(&self, idx: usize) -> bool {
        let Range { start: l, end: r } = ops::column_range(&self.columns, idx);
        l + 1 == r
    }

    /// Write a row which tries to make the corresponding index into a singleton column.
    pub fn try_make_singleton<W: io::Write>(
        &mut self,
        idx: usize,
        writer: &mut DiagramWriter<W, B>,
        diagram_width: usize,
    ) -> io::Result<()> {
        ops::fork_align::<_, _, _, true>(writer, &mut self.columns, idx, ..diagram_width)?;
        writer.write_newline()?;

        Ok(())
    }

    /// Write the annotation following a line
    pub fn write_annotation_line<W: io::Write>(
        &self,
        writer: &mut DiagramWriter<W, B>,
        line: &str,
        alignment: usize,
    ) -> io::Result<()> {
        writer.write_annotation_line(line, alignment, self.config.annotation_margin)
    }

    /// Write annotation which preceds the vertex.
    ///
    /// This method returns the resulting annotation alignment.
    pub fn write_preceding_annotation<'a, W: io::Write>(
        &mut self,
        mut lines: impl Iterator<Item = &'a str>,
        writer: &mut DiagramWriter<W, B>,
    ) -> io::Result<Option<usize>> {
        match lines.next() {
            Some(first_line) => {
                let annotation_alignment = ops::align(writer, &mut self.columns, ..)?;
                writer.write_annotation_line(
                    first_line,
                    annotation_alignment,
                    self.config.annotation_margin,
                )?;
                for line in lines {
                    ops::align(writer, &mut self.columns, ..)?;
                    writer.write_annotation_line(
                        line,
                        annotation_alignment,
                        self.config.annotation_margin,
                    )?;
                }
                Ok(Some(annotation_alignment))
            }
            None => Ok(None),
        }
    }

    /// Write annotation which follows the vertex.
    pub fn write_trailing_annotation<'a, W: io::Write>(
        &mut self,
        mut lines: impl Iterator<Item = &'a str>,
        writer: &mut DiagramWriter<W, B>,
        // the index to prepare for next vertex
        prepare_idx: usize,
        // the max diagram width
        diagram_width: usize,
        // how to align the annotation with the previous call
        annotation_alignment: usize,
    ) -> io::Result<()> {
        // write the first annotation line or a newline
        if let Some(line) = lines.next() {
            writer.write_annotation_line(
                line,
                annotation_alignment,
                self.config().annotation_margin,
            )?;
        } else {
            writer.write_newline()?;
        }

        // prepare space for the next vertex, but don't fork until necessary
        if let Some(mut prev_line) = lines.next() {
            for line in lines {
                ops::fork_align::<_, _, _, false>(
                    writer,
                    &mut self.columns,
                    prepare_idx,
                    ..diagram_width,
                )?;
                writer.write_annotation_line(
                    prev_line,
                    annotation_alignment,
                    self.config.annotation_margin,
                )?;

                prev_line = line;
            }

            ops::fork_align::<_, _, _, true>(
                writer,
                &mut self.columns,
                prepare_idx,
                ..diagram_width,
            )?;
            writer.write_annotation_line(
                prev_line,
                annotation_alignment,
                self.config.annotation_margin,
            )?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_vertex_row<W: io::Write>(
        &mut self,
        writer: &mut DiagramWriter<W, B>,
        next_min_idx: usize,
        l: usize,
        r: usize,
        delay_fork: bool,
        col: usize,
        marker_char: char,
        diagram_width: usize,
    ) -> Result<(), io::Error> {
        if next_min_idx < l {
            // the next minimal index lands before the marker

            let mut offset = if delay_fork {
                ops::fork_align::<_, _, _, false>(
                    writer,
                    &mut self.columns[..l],
                    next_min_idx,
                    ..col,
                )?
            } else {
                ops::fork_align::<_, _, _, true>(
                    writer,
                    &mut self.columns[..l],
                    next_min_idx,
                    ..col,
                )?
            };

            let (actual, next_offset) = ops::marker(writer, marker_char, offset, col)?;
            offset = next_offset;
            if r < self.columns.len() {
                writer.queue_blank(offset.min(self.columns[r].1) - actual);
                ops::align(writer, &mut self.columns[r..], offset..diagram_width)?;
            }
        } else if next_min_idx < r {
            // the next minimal index is a child of the marker

            // first, we use `align` on the preceding columns to make as much space as
            // possible. we can use the unbounded version since `align` by default compacts and this
            // may result in better codegen
            let mut offset = ops::align(writer, &mut self.columns[..l], ..)?;
            let (actual, next_offset) =
                ops::mark_and_prepare(writer, &self.columns, marker_char, offset, next_min_idx)?;
            offset = next_offset;
            if r < self.columns.len() {
                writer.queue_blank(offset.min(self.columns[r].1) - actual);
                ops::align(writer, &mut self.columns[r..], offset..diagram_width)?;
            }
        } else {
            // the next minimal index follows the marker

            let mut offset = ops::align(writer, &mut self.columns[..l], ..)?;
            let (actual, next_offset) = ops::marker(writer, marker_char, offset, col)?;
            offset = next_offset;
            if r < self.columns.len() {
                writer.queue_blank(offset.min(self.columns[r].1) - actual);
                if delay_fork {
                    ops::fork_align::<_, _, _, false>(
                        writer,
                        &mut self.columns[r..],
                        next_min_idx - r,
                        offset..diagram_width,
                    )?;
                } else {
                    ops::fork_align::<_, _, _, true>(
                        writer,
                        &mut self.columns[r..],
                        next_min_idx - r,
                        offset..diagram_width,
                    )?;
                }
            }
        };
        Ok(())
    }

    #[cfg(test)]
    #[allow(unused)]
    fn debug_cols(&self) {
        self.debug_cols_impl(None::<std::convert::Infallible>);
    }

    #[cfg(test)]
    #[allow(unused)]
    fn debug_cols_header<D: std::fmt::Display>(&self, header: D) {
        self.debug_cols_impl(Some(header));
    }

    #[cfg(test)]
    #[allow(unused)]
    fn debug_cols_impl<D: std::fmt::Display>(&self, header: Option<D>) {
        if let Some(min_idx) = self.min_index() {
            if let Some(s) = header {
                println!("{s}:");
            }
            print!(" ->");
            for (i, (vtx, col)) in self.columns.iter().enumerate() {
                if i == min_idx {
                    print!(" ({},*{col})", self.ramifier.marker(&vtx));
                } else {
                    print!(" ({}, {col})", self.ramifier.marker(&vtx));
                }
            }
            println!();
        } else {
            println!("Tree is empty");
        }
    }
}
