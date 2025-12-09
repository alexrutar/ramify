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
    fn new<B>(status: Status, config: &Config<B>) -> Self {
        let ready = if config.lazy {
            status.is_compressed()
        } else {
            status.isolated
        };
        let alignment = status.reserved_width().max(config.min_diagram_width);
        let width = status.width;

        Self {
            alignment,
            width,
            margin: config.annotation_margin,
            ready,
        }
    }

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
}

impl<V, R: TryRamify<V>, B> Columns<V, R, B> {
    /// Get the marker character at the provided index.
    ///
    /// Panics if the index is out of range.
    pub fn marker_char(&self, idx: usize) -> char {
        self.ramifier.marker(&self.columns[idx].0)
    }

    /// Get the column at the provided index.
    ///
    /// Panics if the index is out of range.
    pub fn col(&self, idx: usize) -> usize {
        self.columns[idx].1
    }

    /// Compute the annotation, storing it in the provided buffer.
    pub fn buffer_annotation(&mut self, idx: usize, buf: &mut String) {
        self.ramifier
            .annotate(&self.columns[idx].0, buf)
            .expect("Writing to a `String` should not fail.");
    }

    /// Substitute the vertex at the provided index, replacing it with its children and
    /// recomputing the minimal index.
    pub fn substitute(&mut self, idx: usize) -> Result<(), R::Error> {
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
            .min_by_key(|(_, (e, _))| self.ramifier.key(e))
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
        Ok(RowState::new(col_iter.status(), &self.config))
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
        Ok(RowState::new(col_iter.cols().status(), &self.config))
    }
}
