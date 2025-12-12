//! An abstraction over diagram columns.
//!
//! # TODO Documentation:
//!
//! - introduce basic terminology; vertex; children; parent
//! - idea of 'active' vertices (vertices not yet drawn for which the parent has already been
//!   drawn)
//! - idea of the 'state' being intermediate between two rows
//! - predictive rendering and preparation for following vertices
//! - delayed branching (if there is padding)
//! - width limitations
//! - explain how width interacts with the annotation (we need to make space, so the tree does not
//!   overlap with the annotation in subsequent rows)
//! - one step lookahead, but not more: why lookahead?
//! - fork alignment logic, and 'alignment targets':
//!   - delaying forks until 'inside' the target: once a fork happens, it is much more difficult to move it
//!   - since we do not know the diagram width until the end, we need to compact as much as possible
//! - child order when forking
//! - Why 1-step lookahead?
//!   1. More optimal layout
//!   2. If vertex retrieval fails, we don't want to write any characters.
//!      This is particularly important in `inverted` writing mode.
//! - Implementing 1-step lookahead using the `Shim` trait.
//! - annotation layout, make 'box limit' diagrams showing where the various margins are, etc.
//! - internal data model, i.e. a sorted vec of columns with vertices
//! - description of the fundamental components of the algorithm (basically, operations which
//!   attempt to move a given column to a new location, plus 'forks', and unmoveable markers)
//! ### Internal state
//! The generator corresponds to the state at the `tip` of a partially written branch diagram. In
//! order to reduce the width of the branch diagram, multiple vertices can share the same edges
//! within the diagram.
//!
//! For example, consider the following partial branch diagram. The vertex `0` is the root.
//!
//! We can see that it has children `3`, `1`, and `2`. The vertex `2` also has a child `4`. These
//! vertices also have an unknown number of children that have not yet been drawn, corresponding to the
//! outgoing edges at the bottom of the diagram.
//! ```txt
//! 0
//! ├┬╮
//! │1│
//! ├╮2
//! 3│├╮
//! │││4
//! ```
mod iter;

use std::{iter::repeat, ops::BitOrAssign};

pub use iter::{Alignment, Apply, ColumnIndexIter, ColumnsMut, Gap, Shim, Status};

use crate::{Replacement, TryRamify, writer::Config};

/// The state after writing a row.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct RowState {
    /// The alignment (adjusted based on the configuration).
    alignment: usize,
    /// The width.
    width: usize,
    /// The padding.
    margin: usize,
    /// Whether or not the row is ready for a vertex to be written.
    ready: bool,
}

impl BitOrAssign for RowState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.ready |= rhs.ready;
        self.width = rhs.width;
    }
}

impl RowState {
    pub fn alignment(&self) -> (usize, usize, usize) {
        (self.margin, self.alignment, self.width)
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}

#[derive(Debug)]
pub struct Columns<V, R, B> {
    columns: Vec<(V, usize)>,
    // The first index is the minimal one, and the rest are the equivalent ones.
    // We store it like this because it improves codegen in the acyclic case since
    // the vector will never be mutated and gets dropped.
    //
    // `min_index.is_none()` iff `columns.is_empty()`
    min_index: Option<usize>,
    ramifier: R,
    config: Config<B>,
}

impl<V, R, B> Columns<V, R, B> {
    /// Initialize with a root vertex and the provided ramifier and configuration.
    pub fn init(root: V, ramifier: R, config: Config<B>) -> Self {
        Self {
            columns: vec![(root, 0)],
            ramifier,
            min_index: Some(0),
            config,
        }
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &Config<B> {
        &self.config
    }

    /// Get a mutable reference to the configuration.
    pub fn config_mut(&mut self) -> &mut Config<B> {
        &mut self.config
    }

    /// Get the minimal indices.
    pub fn minimal(&self) -> Option<usize> {
        self.min_index
    }

    /// Returns if there are any remaining active vertices.
    pub fn is_empty(&self) -> bool {
        self.min_index.is_none()
    }

    /// Recover the active vertices.
    pub fn into_active_vertices(self) -> impl ExactSizeIterator<Item = V> {
        self.columns.into_iter().map(|(v, _)| v)
    }

    /// The maximal edge index.
    pub fn max_edge_index(&self) -> Option<usize> {
        self.columns.last().map(|(_, c)| *c)
    }

    /// The number of active vertices.
    pub fn girth(&self) -> usize {
        self.columns.len()
    }

    /// Shrink internal allocations to be as small as possible.
    pub fn shrink_to_fit(&mut self) {
        self.columns.shrink_to_fit();
    }

    /// The row state before the first row has been written.
    pub fn initial_state(&self) -> RowState {
        RowState {
            alignment: 1,
            width: 0,
            margin: self.config.annotation_margin,
            ready: true,
        }
    }

    /// Convert the status to a state report.
    fn state(&self, status: Status) -> RowState {
        let ready = if self.config.minimize_width {
            // we wait until the final column aligns with the target width
            self.max_edge_index()
                .is_none_or(|c| status.isolated && status.target_width == c + 1)
            // dbg!(&status);
            // dbg!(self.max_edge_index());
            // status.isolated
        } else {
            status.isolated
        };
        let alignment = status.reserved_width().max(self.config.min_diagram_width);
        let width = status.width;

        RowState {
            alignment,
            width,
            margin: self.config.annotation_margin,
            ready,
        }
    }
}

impl<V, R, B> Columns<V, R, B> {
    /// Get the marker character at the provided index.
    ///
    /// Panics if the index is out of range.
    pub fn marker_char(&self, idx: usize) -> char
    where
        R: TryRamify<V>,
    {
        self.ramifier.marker(&self.columns[idx].0)
    }

    /// Get the column at the provided index.
    ///
    /// Panics if the index is out of range.
    pub fn col(&self, idx: usize) -> usize {
        self.columns[idx].1
    }

    /// Compute the annotation, storing it in the provided buffer.
    pub fn buffer_annotation(&mut self, idx: usize, buf: &mut String)
    where
        R: TryRamify<V>,
    {
        self.ramifier
            .annotate(&self.columns[idx].0, buf)
            .expect("Writing to a `String` should not fail.");
    }

    /// Substitute the vertex at the provided index, replacing it with its children and
    /// recomputing the minimal index.
    pub fn substitute(&mut self, idx: usize) -> Result<(), R::Error>
    where
        R: TryRamify<V>,
    {
        // in order to optimize substitutions, we temporarily swap indices
        // into the target, and then swap back at the end
        if idx + 1 == self.columns.len() {
            // the minimal index is at the end

            // remove the last element
            let (vtx, col) = self.columns.pop().unwrap();

            // determine the data associated with the element
            let maybe_children = self.ramifier.try_ramify(vtx);

            // FIXME: annoying workaround to deal with borrow checker
            if maybe_children.is_err() {
                let Replacement {
                    value: replacement,
                    err,
                } = unsafe { maybe_children.unwrap_err_unchecked() };
                // put the column back, but with the replacement element
                self.columns.push((replacement, col));

                return Err(err);
            } else {
                let children = unsafe { maybe_children.unwrap_unchecked() };
                // append the new elements
                self.columns.extend(children.into_iter().zip(repeat(col)));
            };
        } else {
            // temporarily swap the minimal element with the last element
            let (vtx, col) = self.columns.swap_remove(idx);

            let maybe_children = self.ramifier.try_ramify(vtx);

            // FIXME: annoying workaround to deal with borrow checker
            if maybe_children.is_err() {
                let Replacement {
                    value: replacement,
                    err,
                } = unsafe { maybe_children.unwrap_err_unchecked() };
                // put the column back with the replacement element
                let last_idx = self.columns.len();
                self.columns.push((replacement, col));
                self.columns.swap(last_idx, idx);

                return Err(err);
            } else {
                let children = unsafe { maybe_children.unwrap_unchecked() };

                // splice onto the swapped last element, inserting the new children
                let last = {
                    let mut iter = self
                        .columns
                        .splice(idx..idx + 1, children.into_iter().zip(repeat(col)));
                    iter.next().unwrap()
                };
                // put the last element back
                self.columns.push(last);
            };
        };

        self.min_index = self
            .columns
            .iter()
            .enumerate()
            .min_by_key(|(_, (e, _))| self.ramifier.sort_key(e))
            .map(|(a, _)| a);

        Ok(())
    }

    /// Write a single row by applying the provided operation to every column.
    pub fn write_row<T, A, E>(&mut self, state: &mut T, op: A) -> Result<RowState, E>
    where
        A: for<'a> Apply<&'a mut T, Error = E> + Copy,
    {
        let mut col_iter = ColumnsMut::new(&mut self.columns, self.min_index.map(|i| (i, &[][..])));
        while col_iter.apply(op, state)?.is_some() {}
        let status = col_iter.status();
        Ok(self.state(status))
    }

    /// Write a single row by applying the provided operation to every column, with a shim at a
    /// specific index.
    pub fn write_shimmed_row<T, A, S, E>(
        &mut self,
        state: &mut T,
        op: A,
        shim: (usize, S),
    ) -> Result<RowState, E>
    where
        A: for<'a> Apply<&'a mut T, Error = E> + Copy,
        S: for<'a> Shim<&'a mut T, Error = E>,
    {
        let mut col_iter = ColumnsMut::new(&mut self.columns, self.min_index.map(|i| (i, &[][..])))
            .with_shim(shim);
        while col_iter.apply(op, state)?.is_some() {}
        let status = col_iter.cols().status();
        Ok(self.state(status))
    }
}
