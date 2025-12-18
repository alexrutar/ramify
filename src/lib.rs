//! # Ramify
//!
//! Ramify is a library for generating *branch diagrams* to visualize hierarchical data.
//! ```txt
//! 0       0        0         0
//! ├┐      ├╮       ├┬┐       ├┬┐
//! 1├┐     1╰╮      │1├┐      │1│
//! │2│     ├╮│      ││2│      2│└─┐
//! │3│     2│├╮     │3││      │└─┐│
//! ├┐│     │││3     │┌┘│      ├┬┐││
//! 4││     ├││╯     ││┌┼┐     │3│││
//!  5│     4││      │││4│     4┌┘││
//! ┌┘6     │5│      ││5┌┘      5┌┘│
//! 7       ├─╯      │6┌┘        6┌┘
//!         6        7┌┘          7
//!                   8
//! ```
//! This library is specifically designed for ordered data: this library generates output similar to
//! `git log --graph --all`, rather than the output of `tree`. A prototypical application is to visualize the
//! undo-tree of a text file. The order is the timestamp of the edit, and the tree structure
//! results from the undo relation.
//!
//! Getting started:
//!
//! - To describe your hierarchical data, implement [`Ramify`] or [`TryRamify`].
//! - To generate the branch diagram itself, use the [`Generator`] struct.
//! - To configure the diagram layout and appearance, use the [`Config`] struct or the
//!   [`branch_writer!`] macro. Read more in the [`writer`] module.
//!
//! ## Usage examples
//!
//! Usage examples can be found in the [examples
//! folder](https://github.com/alexrutar/ramify/tree/master/examples) on GitHub.

#![deny(missing_docs)]

pub(crate) mod columns;
mod layout;
pub mod writer;

use std::convert::Infallible;

pub use self::{
    layout::{Generator, WriteVertexError},
    writer::Config,
};

/// A trait representing hierarchical data structures with efficient iteration of children.
///
/// For a version of this trait in which iteration of children might fail, see [`TryRamify`].
///
/// This trait includes provided methods and default methods.
///
/// Also see the [`Generator`] documentation for more information, particularly concerning [the sequence of method calls](Generator#method-call-guarantees) and [resource mangement](Generator#resource-management).
pub trait Ramify<V> {
    /// Iterate over the children of the vertex.
    ///
    /// This method is called exactly once for each vertex immediately before writing the
    /// corresponding branch diagram row.
    ///
    /// # Iteration order
    ///
    /// The iteration order is used to determine the horizontal order in which the vertices are
    /// drawn in the tree. This need not correspond to the precise column in which the node is
    /// actually drawn.
    ///
    /// The below diagram shows the impact of various orders on how the nodes are laid out, for a
    /// node with key `0`, which has children with keys `1 2 3` iterated in various orders.
    /// ```txt
    /// 123  132  213  231  312  321
    ///
    /// 0    0    0    0    0    0
    /// ├╮   ├╮   ├┬╮  ├╮   ├┬╮  ├╮
    /// 1│   1│   │1│  │1   │1│  │1
    /// ╭┤   ╭┤   2╭╯  ├╮   │ 2  ├╮
    /// 2│   │2    3   2│   3    │2
    ///  3   3          3        3
    /// ```
    /// Iterating in sorted order (either increasing or decreasing) or otherwise guaranteeing that
    /// the minimal element is first or last tends to produce narrower trees since this avoids 3-way forks.
    fn ramify(&mut self, vtx: V) -> impl IntoIterator<Item = V>;

    /// Get the sort key associated with a vertex.
    ///
    /// This key is used for the *vertical* render order; that is, to decide which vertex should be
    /// rendered next. This is different than the iteration order of the children. See
    /// the documentation for [`Ramify::ramify`] to compare.
    ///
    /// The active vertices are passed to [`Iterator::min_by_key`] when deciding which vertex
    /// should be rendered next on each iteration. In particular, the first element is returned if
    /// several elements are equally minimal.
    ///
    /// The key is used ephemerally for sorting purposes and is not stored within the branch
    /// diagram. In particular, this method could be called many times for a given vertex.
    ///
    /// # Key order
    ///
    /// The keys are drawn in increasing order.
    /// Use [`Reverse`](std::cmp::Reverse) or a custom [`Ord`] implementation if the vertices in your
    /// tree should be arranged in decreasing order.
    ///
    /// In many standard use-cases, the children of a vertex are greater than the
    /// vertex itself. However, failing to guarantee this will not corrupt the branch diagram.
    /// The next vertex which is drawn is simply the minimal vertex out of the *active vertices* (the vertices with an immediate parent already drawn to the diagram).
    fn sort_key(&self, vtx: &V) -> impl Ord;

    /// The vertex marker in the branch diagram.
    ///
    /// The marker is the character written inside the branch diagram.
    /// In the below diagrams, the vertex markers are the chars `0`, `1`, `2`, and `3`.
    /// ```txt
    /// 0
    /// ├┬╮
    /// │1│
    /// 2╭╯
    ///  3
    /// ```
    ///
    /// # Char width
    ///
    /// This should be a char with width exactly 1 when displayed to the terminal. Other characters,
    /// such as control characters or double-width characters (mainly those described in
    /// [Unicode Annex #11](https://www.unicode.org/reports/tr11/tr11-11.html)) will corrupt the
    /// tree drawing.
    ///
    /// Here are some characters which might be useful:
    ///
    /// - `*` (`\u{002a}`)
    /// - `◊` (`\u{25ca}`)
    /// - `✕` (`\u{2715}`)
    /// - `◈` (`\u{25c8}`)
    /// - `◉` (`\u{25c9}`)
    fn marker(&self, vtx: &V) -> char;

    /// An annotation to write alongside a vertex.
    ///
    /// The buffer is cleared before it is passed to this method.
    ///
    /// This will be called exactly once per vertex.
    /// The lines in the buffer are written sequentially, with the first line written on the
    /// same line as the vertex with which it is associated. The default implementation
    /// does not write an annotation.
    ///
    /// # Implementation details
    ///
    /// Implementations of this method should write the annotation directly into the buffer,
    /// including newlines for annotations spanning multiple lines. The annotations are
    /// automatically line-broken and aligned with the branch diagram when rendered.
    ///
    /// Like the standard library implementation of [`str::lines`](str#method.lines), the final
    /// trailing newline is optional and ignored if present. If you want extra space between
    /// consecutive annotations, it is best to use the [`row_padding`](Config::row_padding)
    /// option of the [`Config`] struct.
    ///
    /// # Example
    ///
    /// The presence of the annotation influences the drawing of the tree, in that subsequent
    /// vertices are delayed in order to make space for the entire annotation followed by the
    /// margin.
    /// ```txt
    /// 0   An annotation occupying two lines
    /// ╰╮  followed by one line of margin
    /// ╭┼╮
    /// │1│ An annotation with one line and no margin.
    /// 2╭╯
    ///  3  The annotation for vertex 2 is empty.
    /// ```
    #[allow(unused)]
    #[inline]
    fn annotate(&self, vtx: &V, buf: &mut String) {}

    /// Return if two vertices are identical and therefore should be merged.
    ///
    /// The first argument is the current minimal vertex and the second argument is a different active
    /// vertex. This method is called once for other active vertex after computing the new minimal
    /// vertex.
    ///
    /// The default implementation always returns `false`, so vertices will never be merged.
    /// Vertices which are merged will be passed to [`cleanup`](Ramify::cleanup).
    ///
    /// # Difference from [`sort_key`](Ramify::sort_key)
    ///
    /// Unlike [`sort_key`](Ramify::sort_key), this method should check that the
    /// vertices are exactly the same. What this means depends on the vertex type, but
    /// this might look like [`Rc::ptr_eq`](std::rc::Rc::ptr_eq), or like comparison of a `usize`
    /// index for flattened graph-like structures, or comparison of uniquely-defining metadata
    /// (like a Git commit hash).
    ///
    /// # Implementation requirements
    ///
    /// This method must be compatible with [`Ramify::sort_key`]: **if two vertices are identical,
    /// then their sort keys must also be equal**. The converse need not hold (the sort
    /// keys can be equal even if the keys are not identical). Failure to uphold this invariant
    /// will result in otherwise identical vertices not being merged.
    ///
    /// Note that this *only checks active vertices*. If there is an identical vertex but it is an
    /// offspring of a vertex which has yet to be written, this vertex will not be merged.
    ///
    /// If every child of a vertex is strictly larger than the vertex itself (as ordered by
    /// [`sort_key`](Ramify::sort_key)) and the above compatibility requirement is upheld, it is
    /// guaranteed that identical vertices will not be missed.
    #[allow(unused)]
    #[inline]
    fn is_identical(&self, vtx: &V, other: &V) -> bool {
        false
    }

    /// Clean up a merged vertex.
    ///
    /// If [`is_identical`](Ramify::is_identical) returns `true`, the `other` vertex will be removed from the
    /// list of active vertices. When it is removed, it is passed to this method.
    ///
    /// The default implementation drops the vertex.
    #[inline]
    fn cleanup(&mut self, vtx: V) {
        drop(vtx);
    }
}

/// The error returned when a ramifier fails to determine the children associated with
/// a vertex.
///
/// This struct is used as the error variant returned by [`TryRamify::try_ramify`]. See those
/// docs for more detail.
#[derive(Debug)]
pub struct Failed<P, E = ()> {
    /// A placeholder vertex for retry attempts.
    pub placeholder: P,
    /// The associated error.
    pub err: E,
}

impl<P, E: Default> From<P> for Failed<P, E> {
    fn from(placeholder: P) -> Self {
        Self {
            placeholder,
            err: E::default(),
        }
    }
}

/// Try to iterate over the children of the vertex.
///
/// This is a fallible version of [`Ramify`] where the call to [`Ramify::ramify`] might fail.
/// This trait instead has a method [`TryRamify::try_ramify`], which can either return a list of
/// children, or fail and return a replacement vertex.
///
/// The [`Ramify`] docs contain much more detail. Here, we only document the differences.
///
/// ### Blanket implementation
///
/// There is a blanket implementation of `TryRamify<V>` whenever a type is `Ramify<V>` with the
/// call to [`try_ramify`](TryRamify::try_ramify) always returning `Ok(_)`. In particular, you can use
/// a [`Ramify`] implementation anywhere a [`TryRamify`] implementation is expected.
pub trait TryRamify<V> {
    /// An error which may occur while trying to retrieve the children.
    type Error;

    /// A placeholder passed to the next render attempt.
    type Placeholder;

    /// Try to iterate over the children of the vertex.
    ///
    /// If it is not possible to determine the children, a placeholder must be returned in the `Err(_)`
    /// variant. The placeholder will be passed to [`retry_ramify`](TryRamify::retry_ramify) for subsequent
    /// attempts.
    ///
    /// The marker character and the annotation of the previous vertex are used, regardless
    /// of the returned replacement vertex. The replacement vertex is only used to pass
    /// additional state onwards to the next call of this method.
    ///
    /// # Common implementation patterns
    ///
    /// Here are a few common patterns for which this method is designed.
    ///
    /// 1. *Permanent failure*: This method can be used to abort on an unrecoverable failure. Since the error is
    ///    propagated to the caller, the caller can use this to abort iteration permanently. In this
    ///    case, the placeholder in the `Err(_)` variant would just be the unit enum since it is
    ///    unused anyway.
    /// 2. *Temporary failure*: If the failure is temporary, the original vertex can be returned in
    ///    the `Err(_)` variant and the caller can wait (or do something else) before attempting
    ///    to write a vertex row again. In this case, `retry` is just a call to `try_ramify`.
    /// 3. *Local failure*: If only this specific vertex cannot be written (whereas it is
    ///    reasonable for iteration to continue with the other vertices), this method should succeed
    ///    but return a single special vertex which can be used later to report the failure inside
    ///    the tree itself. In this case, one might implement [`Ramify`] instead.
    fn try_ramify(
        &mut self,
        vtx: V,
    ) -> Result<impl IntoIterator<Item = V>, Failed<Self::Placeholder, Self::Error>>;

    /// Try to iterate over the children of a vertex when the previous attempt failed.
    ///
    /// Iteration may fail multiple times, in which case the placeholder contained in the
    /// [`Failed`] struct from the previous attempt will be passed to the subsequent attempt.
    fn retry_ramify(
        &mut self,
        prev: Self::Placeholder,
    ) -> Result<impl IntoIterator<Item = V>, Failed<Self::Placeholder, Self::Error>>;

    /// Get the sort key associated with a vertex.
    fn sort_key(&self, vtx: &V) -> impl Ord;

    /// The vertex marker in the branch diagram.
    fn marker(&self, vtx: &V) -> char;

    /// An annotation to write alongside a vertex.
    #[allow(unused)]
    fn annotate(&self, vtx: &V, buf: &mut String) {}

    /// Determine if two vertices are identical and should be merged.
    #[allow(unused)]
    #[inline]
    fn is_identical(&self, vtx: &V, other: &V) -> bool {
        false
    }

    /// Clean up a merged vertex.
    #[inline]
    fn cleanup(&mut self, vtx: V) {
        drop(vtx);
    }
}

impl<R: Ramify<V>, V> TryRamify<V> for R {
    type Error = Infallible;

    type Placeholder = Infallible;

    fn try_ramify(
        &mut self,
        vtx: V,
    ) -> Result<impl IntoIterator<Item = V>, Failed<Self::Placeholder, Self::Error>> {
        Ok(<Self as Ramify<V>>::ramify(self, vtx))
    }

    fn retry_ramify(
        &mut self,
        _: Self::Placeholder,
    ) -> Result<impl IntoIterator<Item = V>, Failed<Self::Placeholder, Self::Error>> {
        // this is unreachable since `Self::Retry` is uninhabited
        Ok(std::iter::empty())
    }

    fn sort_key(&self, vtx: &V) -> impl Ord {
        <Self as Ramify<V>>::sort_key(self, vtx)
    }

    fn marker(&self, vtx: &V) -> char {
        <Self as Ramify<V>>::marker(self, vtx)
    }

    fn annotate(&self, vtx: &V, buf: &mut String) {
        <Self as Ramify<V>>::annotate(self, vtx, buf)
    }

    fn is_identical(&self, vtx: &V, other: &V) -> bool {
        <Self as Ramify<V>>::is_identical(self, vtx, other)
    }

    fn cleanup(&mut self, vtx: V) {
        <Self as Ramify<V>>::cleanup(self, vtx)
    }
}
