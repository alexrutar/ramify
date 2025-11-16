//! # Ramify
//!
//! Ramify is a library for generating *branch diagrams* for heirarchical data.
//! ```txt
//! 0       0
//! ├╮      ├┬╮
//! 1├╮     │1│
//! │2│     2│╰─╮
//! │3│     │╰─╮│
//! ├╮│     ├┬╮││
//! 4││     │3│││
//!  5│     4╭╯││
//! ╭╯6      5╭╯│
//! 7         6╭╯
//!            7
//! ```
//! This library is specifically designed for ordered data: this is closer to the output of
//! `git log --graph --all` than the output of `tree`. A prototypical application is to visualize the
//! undo-tree of a text file. The order is the timestamp of the edit, and the tree structure
//! results from the undo relation.
//!
//! The visualization is generic over ordered heirarchical data with efficient
//! iteration over immediate children. See the [`Ramify`] trait for more detail.
//!
//! ## Basic examples
pub mod config;
mod layout;
mod writer;

use std::fmt::Display;

pub use self::{layout::BranchDiagram, writer::DiagramWriter};

/// Heirarchical types with efficient iteration of children.
///
/// The type `V` is a pointer-like type implementation should use to keep track of the posititon
/// within the data.
///
/// If you are using a recursive tree type like
/// ```
/// struct Vertex<T> {
///     data: T,
///     children: Vec<Vertex<T>>
/// }
/// ```
/// then `V` is probably a reference `&'t Vertex`. If your data is stored in some sort of flat data structure, then `V` is
/// perhaps an index like `usize`.
pub trait Ramify<V> {
    /// The data by which the vertices should be sorted.
    ///
    /// The keys are drawn in increasing order.
    /// Use [`Reverse`](std::cmp::Reverse) or a custom [`Ord`] implementation if the keys in your
    /// tree are decreasing instead.
    type Key: Ord;

    /// Iterate over the children of the vertex.
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
    /// Iterating in sorted order (either increasing or decreasing) tends to produce narrower
    /// trees since this avoids 3-way forks.
    fn children(&self, vtx: V) -> impl Iterator<Item = V>;

    fn get_key(&self, vtx: V) -> Self::Key;

    /// The vertex marker to use for the marker in the branch drawing.
    ///
    /// This should be a char with width exactly `1` when displayed to the terminal. Other characters,
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
    fn marker(&self, vtx: V) -> char;

    /// An annotation to write alongside a vertex.
    ///
    /// The annotation is written in the same line as the provided vertex.
    ///
    /// The `tree_width` argument can be used for more fine-grained alignment. It is the maximum
    /// over the widths of all lines in the tree diagram before the next vertex is drawn.
    ///
    /// Returning `None` indicates that there is no annotation to be written. This will prevent
    /// unnecessary whitespace from being written. The default implementation always
    /// returns `None`.
    #[allow(unused)]
    fn annotation(&self, vtx: V, tree_width: usize) -> Option<impl Display> {
        None::<std::convert::Infallible>
    }
}
