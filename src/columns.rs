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
//! - delayed vertex vs normal vertex mode:
//!   - if the vertex is written last, we do not branch it at all (have to wait until after it is
//!     written) especially to avoid breaking the annotation after we have already started writing it
//! - delayed vertex mode is useful if your annotations only have exactly one line
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

use std::{
    iter::{Map, repeat},
    vec::IntoIter,
};

pub use iter::{Alignment, Apply, ColumnsMut, MinIndices, Position, Shim, Status};

use crate::{Config, TryRamify};

/// The state after writing a row.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct RowState {
    /// The alignment (adjusted based on the configuration).
    pub(crate) alignment: usize,
    /// The width.
    pub(crate) width: usize,
    /// Whether every minimal index is isolated.
    isolated: bool,
    /// Whether or not the row is ready for a vertex to be written.
    ready: bool,
}

impl RowState {
    pub fn update(&mut self, other: &RowState) {
        self.isolated = other.isolated;
        self.ready = other.ready;
        self.width = other.width;
    }

    //     pub fn alignment(&self) -> (usize, usize, usize) {
    //         (self.margin, self.alignment, self.width)
    //     }

    pub fn is_isolated(&self) -> bool {
        self.isolated
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}

pub struct DebugCols<'a, V, R: TryRamify<V>>(&'a Columns<V, R>);

impl<'a, V, R: TryRamify<V>> std::fmt::Debug for DebugCols<'a, V, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self.0
                    .columns
                    .iter()
                    .map(|(v, c)| (self.0.ramifier.marker(v), c)),
            )
            .finish()
    }
}

struct DebugMinimal<'a, V, R: TryRamify<V>>(&'a Columns<V, R>);

impl<'a, V, R: TryRamify<V>> std::fmt::Debug for DebugMinimal<'a, V, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(idx) = self.0.min_index {
            std::fmt::Debug::fmt(&Some((idx, &self.0.equivalent_to_min)), f)
        } else {
            std::fmt::Debug::fmt(&None::<()>, f)
        }
    }
}

struct DebugActive<'a, V, R: TryRamify<V>>(&'a Columns<V, R>);

impl<'a, V, R: TryRamify<V>> std::fmt::Debug for DebugActive<'a, V, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Active")
            .field("columns", &DebugCols(self.0))
            .field("minimal", &DebugMinimal(self.0))
            .finish()
    }
}

pub struct SuspendedColumns<V, R> {
    inner: Columns<V, R>,
    min_index: usize,
    col: usize,
    marker: char,
}

type ActiveVertices<V> = Map<IntoIter<(V, usize)>, fn((V, usize)) -> V>;

impl<V, R> SuspendedColumns<V, R> {
    /// Recover the active vertices.
    pub fn into_active_vertices(self) -> ActiveVertices<V> {
        self.inner.into_active_vertices()
    }

    /// Resume iteration by manually specifying new children.
    pub fn resume<I>(mut self, children: I) -> (Columns<V, R>, usize, char)
    where
        I: IntoIterator<Item = V>,
        R: TryRamify<V>,
    {
        let (col, m) = {
            if self.min_index == self.inner.columns.len() {
                // append the new elements
                self.inner
                    .columns
                    .extend(children.into_iter().zip(repeat(self.col)));
            } else {
                let last = {
                    let mut iter = self.inner.columns.splice(
                        self.min_index..self.min_index + 1,
                        children.into_iter().zip(repeat(self.col)),
                    );
                    iter.next().unwrap()
                };
                // put the last element back
                self.inner.columns.push(last);
            }
            (self.col, self.marker)
        };

        self.inner.recompute_minimal();

        (self.inner, col, m)
    }
}

#[derive(Debug)]
pub struct Columns<V, R> {
    columns: Vec<(V, usize)>,
    // The first index is the minimal one, and the rest are the equivalent ones.
    // We store it like this because it improves codegen in the acyclic case since
    // the vector will never be mutated and gets dropped.
    //
    // `min_index.is_none()` iff `columns.is_empty()`
    min_index: Option<usize>,
    // if `try_ramify` fails, store the key, the column, and the marker char
    // failed_placeholder: Option<(P, usize, char)>,
    equivalent_to_min: Vec<usize>,
    ramifier: R,
    config: Config,
}

impl<V, R> Columns<V, R> {
    /// Initialize with a root vertex and the provided ramifier and configuration.
    pub fn init(root: V, ramifier: R, config: Config) -> Self {
        Self {
            columns: vec![(root, 0)],
            ramifier,
            min_index: Some(0),
            // failed_placeholder: None,
            equivalent_to_min: Vec::new(),
            config,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> Config {
        self.config
    }

    /// Get a mutable reference to the internal configuration.
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Returns if there are any remaining active vertices.
    pub fn is_empty(&self) -> bool {
        self.min_index.is_none()
    }

    pub fn is_merged(&self) -> bool {
        self.equivalent_to_min.is_empty()
    }

    /// Recover the active vertices.
    pub fn into_active_vertices(self) -> ActiveVertices<V> {
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
        self.equivalent_to_min.shrink_to_fit();
    }

    /// Convert the status to a state report.
    fn state(&self, status: Status) -> RowState {
        let ready = self.equivalent_to_min.is_empty()
            && if self.config.minimize_width {
                // we wait until the final column aligns with the target width
                self.max_edge_index()
                    .is_none_or(|c| status.isolated && status.target_width == c + 1)
            } else {
                status.isolated
            };
        let alignment = status.reserved_width();
        let width = status.width;

        RowState {
            alignment,
            width,
            isolated: status.isolated,
            ready,
        }
    }

    #[allow(unused)]
    pub fn debug_active(&self) -> impl std::fmt::Debug
    where
        R: TryRamify<V>,
    {
        DebugActive(self)
    }
}

impl<V, R> Columns<V, R> {
    /// Substitute the vertex at the minimal index, replacing it with its children and
    /// recomputing the minimal index. Returns `None` if there are no columns.
    ///
    /// This returns the column at the index as well as the corresponding marker, and writes the
    /// annotation to the provided buffer.
    #[allow(clippy::type_complexity)]
    pub fn try_substitute(
        mut self,
        buf: &mut String,
    ) -> Result<(Self, Option<(usize, char)>), (SuspendedColumns<V, R>, R::Error)>
    where
        R: TryRamify<V>,
    {
        let Some(idx) = self.min_index else {
            return Ok((self, None));
        };

        let ret = if idx + 1 == self.columns.len() {
            // pop the minimal element
            let (vtx, col) = self.columns.pop().unwrap();
            let marker = self.ramifier.marker(&vtx);
            buf.clear();
            self.ramifier.annotate(&vtx, buf);

            // determine the data associated with the element
            let res = self.ramifier.try_ramify(vtx).map(|children| {
                self.columns.extend(children.into_iter().zip(repeat(col)));
                (col, marker)
            });

            match res {
                Ok(t) => t,
                Err(err) => {
                    let failed = SuspendedColumns {
                        inner: self,
                        min_index: idx,
                        col,
                        marker,
                    };
                    return Err((failed, err));
                }
            }
        } else {
            // swap the minimal element with the last element
            let (vtx, col) = self.columns.swap_remove(idx);
            let marker = self.ramifier.marker(&vtx);
            buf.clear();
            self.ramifier.annotate(&vtx, buf);
            let res = self.ramifier.try_ramify(vtx).map(|children| {
                // splice onto the swapped last element, inserting the new children
                let last = {
                    let mut iter = self
                        .columns
                        .splice(idx..idx + 1, children.into_iter().zip(repeat(col)));
                    iter.next().unwrap()
                };
                // put the last element back
                self.columns.push(last);
                (col, marker)
            });

            match res {
                Ok(t) => t,
                Err(err) => {
                    let failed = SuspendedColumns {
                        inner: self,
                        min_index: idx,
                        col,
                        marker,
                    };
                    return Err((failed, err));
                }
            }
        };

        self.recompute_minimal();

        Ok((self, Some(ret)))
    }

    /// Recompute the minimal index after iteration.
    fn recompute_minimal(&mut self)
    where
        R: TryRamify<V>,
    {
        // recompute the minimal index
        self.min_index = self
            .columns
            .iter()
            .enumerate()
            .min_by_key(|(_, (e, _))| self.ramifier.sort_key(e))
            .map(|(a, _)| a);

        // find equivalent indices if necessary
        if let Some(min_idx) = self.min_index {
            self.equivalent_to_min.clear();
            for idx in min_idx + 1..self.columns.len() {
                if self
                    .ramifier
                    .is_identical(&self.columns[min_idx].0, &self.columns[idx].0)
                {
                    self.equivalent_to_min.push(idx)
                }
            }
        }
    }

    /// Get a mutable column iterator holding the minimal indices.
    fn columns_mut(&mut self) -> ColumnsMut<'_, V> {
        ColumnsMut::new(
            &mut self.columns,
            self.min_index.map(|i| (i, &self.equivalent_to_min[..])),
        )
    }

    /// Write a single row by applying the provided operation to every column.
    pub fn write_row<T, A, E>(&mut self, state: &mut T, op: A) -> Result<RowState, E>
    where
        A: for<'a> Apply<&'a mut T, Error = E> + Copy,
    {
        let mut col_iter = self.columns_mut();
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
        let mut col_iter = self.columns_mut().with_shim(shim);
        while col_iter.apply(op, state)?.is_some() {}
        let status = col_iter.cols().status();
        Ok(self.state(status))
    }

    /// Clear all vertices which are equivalent to the minimal vertex.
    fn clear_equivalent(&mut self)
    where
        R: TryRamify<V>,
    {
        let mut i = 0;
        let mut min_i = 0;
        self.columns.retain(|_| {
            if self.equivalent_to_min.get(min_i).is_some_and(|m| *m == i) {
                i += 1;
                min_i += 1;
                false
            } else {
                i += 1;
                true
            }
        });

        self.equivalent_to_min.clear();
    }

    /// Write a single row by applying the provided operation to every column, and then deleting
    /// the merged indices.
    pub fn write_merge_row<T, A, E>(&mut self, state: &mut T, op: A) -> Result<RowState, E>
    where
        A: for<'a> Apply<&'a mut T, Error = E> + Copy,
        R: TryRamify<V>,
    {
        let mut col_iter = self.columns_mut();
        while col_iter.apply(op, state)?.is_some() {}
        let status = col_iter.status();
        self.clear_equivalent();
        Ok(self.state(status))
    }
}
