//! # Ramify
//!
//! Ramify is a library for generating *branch diagrams* for heirarchical data.
//! ```txt
//! 0       0         0    
//! ├╮      ├┬╮       ├┬╮  
//! 1├╮     │1├╮      │1│  
//! │2│     ││2│      2│╰─╮
//! │3│     │3││      │╰─╮│
//! ├╮│     │╭╯│      ├┬╮││
//! 4││     ││╭┼╮     │3│││
//!  5│     │││4│     4╭╯││
//! ╭╯6     ││5╭╯      5╭╯│
//! 7       │6╭╯        6╭╯
//!         7╭╯          7
//!          8             
//! ```
//! This library is specifically designed for ordered data: this is closer to the output of
//! `git log --graph --all` than the output of `tree`. A prototypical application is to visualize the
//! undo-tree of a text file. The order is the timestamp of the edit, and the tree structure
//! results from the undo relation.
//!
//! Key features:
//!
//! - Memory efficient streaming implementation: new vertices are not requested until the
//!   parent vertex has been rendered.
//! - Robust support for metadata via annotations.
//! - Generic over ordered heirarchical data with efficient iteration over immediate children. See
//!   the [`Ramify`] trait for more detail.
//!
//! ## Basic examples
//! TODO: write
//!
//! - fancy examples? spanning tree of a graph; vertex weights are visitation order (breadth first
//!   search); like a* algorithm
//! - 'ramify' allowed to take mutable self-reference?

#![deny(missing_docs)]

mod layout;
pub mod writer;

use std::fmt;

pub use self::{layout::Generator, writer::Config};

/// A trait representing heirarchical data structures with efficient iteration of children.
///
/// ### Vertex type `V`
/// The type `V` is a pointer-like type that the implementation should use to keep track of the posititon
/// within the data.
///
/// If you are using a recursive tree type like
/// ```
/// struct Vtx<T>(T, Vec<Vtx<T>>);
/// ```
/// then `V` is perhaps a reference `&'t Vtx`. If your data is stored in some sort of flat data structure, then `V` is
/// perhaps an index like `usize`.
///
/// ### Method calls when driven by a [`Generator`]
///
/// When a [`Ramify`] implementation is used by a [`Generator`], the following calls are made
/// when rendering a row and its annotation (a single call to
/// [`write_next_vertex`](Generator::write_next_vertex)).
///
/// - [`Ramify::marker`] is called exactly once to determine the diagram marker for the minimal vertex.
/// - [`Ramify::annotation`] is called exactly once called to determine the annotation for the
///   minimal vertex.
/// - [`Ramify::children`] is called exactly once to replace the current minimal vertex with its
///   children
/// - [`Ramify::get_key`] is called once for every active vertex (including the children of the
///   previous vertex) to determine the new minimal vertex.
///
/// Moreover, the call to [`Ramify::children`] is **guaranteed to be last** for each vertex. This is enforced by the borrow checker since the signature takes ownership of `V`.
/// The other methods only take a reference to the vertex rather than receive the vertex itself.
///
/// Otherwise, the relative order between these calls, and moreover the order relative to writes, is unspecified.
pub trait Ramify<V> {
    /// The key by which the vertices should be sorted.
    ///
    /// See [`Ramify::get_key`] for more detail.
    type Key: Ord;

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
    fn children(&mut self, vtx: V) -> impl Iterator<Item = V>;

    /// Get the key associated with a vertex.
    ///
    /// This key is used for the *vertical* render order; that is, to decide which vertex should be
    /// rendered next. This is different than the iteration order of the children. See
    /// the documentation for [`Ramify::children`] to compare.
    ///
    /// The active vertices are passed to [`Iterator::min_by_key`] when deciding which vertex
    /// should be rendered next on each iteration. In particular, the first element is returned if
    /// several elements are equally minimum.
    ///
    /// The key is used ephemerally for sorting purposes and is not stored within the branch
    /// diagram. In particular, this method could be callled many times for a given vertex.
    ///
    /// # Key order
    ///
    /// The keys are drawn in increasing order.
    /// Use [`Reverse`](std::cmp::Reverse) or a custom [`Ord`] implementation if the vertices in your
    /// tree should be arranged in decreasing order.
    ///
    /// In many standard use-cases, the children of a vertex are greater than the
    /// vertex itself. However, failing to guarantee this will not corrupt the branch diagram.
    /// The next vertex which is drawn is simply the minimal vertex out of the *active vertices* (the vertices vertices with an immediate parent already drawn to the diagram).
    fn get_key(&self, vtx: &V) -> Self::Key;

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
    /// This will be called exactly once per vertex, after its children have been computed via a
    /// call to the
    /// [`children`](Ramify::children) method.
    ///
    /// The lines of the annotations are written sequentially, with the first line written on the
    /// same line as the vertex with which it is associated.
    ///
    /// The default implementation does not write an annotation.
    ///
    /// # Implementation details
    ///
    /// Implementations of this method should write the annotation directly into the buffer,
    /// including newlines for annotations spanning multiple lines. The annotations are
    /// automatically line-broken and aligned with the branch diagram when rendered.
    ///
    /// Like the standard library implementation of [`str::lines`](str#method.lines), the final
    /// trailing newline is optional and ignored if present. If you want padding between your
    /// annotations, it is better use the
    /// [`margin_below`](Config::margin_below) option of the
    /// [`Config`] struct.
    ///
    /// The `offset` argument is the number of characters that will be written before the
    /// annotation is drawn. This includes any extra spaces requested in
    /// [`margin_left`](Config::margin_left). This number can be used to prevent writing
    /// very long lines to output if the tree is very wide.
    ///
    /// # Example
    ///
    /// The presence of the annotation influences the drawing of the tree, in that subsequent
    /// vertices are delayed in order to make space for the entire annotation followed by the
    /// margin.
    /// ```txt
    /// 0   An annotation occupying two lines
    /// ╰╮  followed by one line of padding
    /// ╭┼╮
    /// │1│ An annotation with one line and no padding.
    /// 2╭╯
    ///  3  The annotation for vertex 2 is empty.
    /// ```
    #[allow(unused)]
    fn annotation<B: fmt::Write>(&self, vtx: &V, buf: B) -> fmt::Result {
        Ok(())
    }
}
