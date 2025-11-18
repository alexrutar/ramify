//! # Configuration and styles for the writer
//!
//! This module contains the [`Config`] struct, which defines the configuration used by the
//! [`Generator`](crate::Generator) when drawing the branch diagram.
//!
//! The [`Config`] struct uses compile-time specification of the drawing style. You can implement
//! your own; see the [`WriteBranch`] trait. An implementation of this trait in simple cases can be
//! auto-generated using the [`branch_writer`] macro.
//!
//! Follow the above documentation links for more detail. Just below you will find a [style gallery](#style-gallery).
//! Beyond that there is detailed documentation concerning the [layout
//! algorithm](#layout-algorithm-documentation).
//!
//! ## Style gallery
//! Here is a gallery of the various default lines styles.
//! ```txt
//! rounded  sharp    rounded     sharp       doubled
//! corners  corners  corners     corners     lines
//!                   wide        wide        (only wide)
//!
//!  0        0        0           0           0
//!  ├┬╮      ├┬┐      ├─┬─╮       ├─┬─┐       ╠═╦═╗
//!  │1│      │1│      │ 1 │       │ 1 │       ║ 1 ║
//!  2│╰╮     2│└┐     2 │ ╰─╮     2 │ └─┐     2 ║ ╚═╗
//!  │╰╮│     │└┐│     │ ╰─╮ │     │ └─┐ │     ║ ╚═╗ ║
//!  ├╮││     ├┐││     ├─╮ │ │     ├─┐ │ │     ╠═╗ ║ ║
//!  3│││     3│││     3 │ │ │     3 │ │ │     3 ║ ║ ║
//!  ╭╯││     ┌┘││     ╭─╯ │ │     ┌─┘ │ │     ╔═╝ ║ ║
//!  │╭┤│     │┌┤│     │ ╭─┤ │     │ ┌─┤ │     ║ ╔═╣ ║
//!  │4││     │4││     │ 4 │ │     │ 4 │ │     ║ 4 ║ ║
//!  5╭╯│     5┌┘│     5 ╭─╯ │     5 ┌─┘ │     5 ╔═╝ ║
//!   6╭┤      6┌┤       6 ╭─┤       6 ┌─┤       6 ╔═╣
//!    7│       7│         7 │         7 │         7 ║
//!     8        8           8           8           8
//! ```
//!
//! ## Layout algorithm documentation
//! TODO: not written
//!
//! Explain:
//!
//! - introduce basic terminology; vertex; children; parent
//! - idea of 'active' vertices (vertices not yet drawn for which the parent has already been
//!   drawn)
//! - idea of the 'state' being intermediate between two rows
//! - predictive rendering and preparation for following vertices
//! - delayed branching (if there is padding)
//! - width limitations (how it relates to slack, and a minimum slack parameter)
//! - the algorithm used to compute how much width is required
//! - explain how width interacts with the annotation (we need to make space, so the tree does not
//!   overlap with the annotation in subsequent rows)
//! - one step lookahead, but not more
//! - 2-way vs 3-way forks; child order
//! - annotation layout, make 'box limit' diagrams showing where the various margins are, etc.
//! - internal data model, i.e. a sorted vec of columns with vertices
//! - description of the fundamental components of the algorithm (basically, operations which
//!   attempt to move a given column to a new location, plus 'forks', and unmoveable markers)
//! - whitespace management; no trailing whitespace; buffered whitespace (explain how this relates
//!   to [`WriteBranch`]).
//! - children having mutable self-reference, but none of the other methods
///
/// ### Internal state
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
/// TODO: write more
mod branch;

pub use self::branch::{__branch_writer_impl, Branch, branch_writer};

use std::{fmt, io, marker::PhantomData};

/// Configuration passed to a [`Generator`](crate::Generator) to control the appearance
/// and layout of the branch diagram and associated annotations.
#[derive(Debug, Clone)]
pub struct Config<B = RoundedCorners> {
    /// The margin between each annotation. The default is `0`.
    pub(crate) margin_below: usize,
    /// The margin between the annotation and the branch diagram. The default is `1`.
    pub(crate) margin_left: usize,
    branch_writer: PhantomData<B>,
}

impl<B> Default for Config<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> Config<B> {
    /// Initialize using default values.
    ///
    /// Calling this method requires type annotations.
    /// Also see the convenience methods:
    ///
    /// - [`with_rounded_corners`](Self::with_rounded_corners)
    /// - [`with_rounded_corners_wide`](Self::with_rounded_corners_wide)
    /// - [`with_sharp_corners`](Self::with_sharp_corners)
    /// - [`with_sharp_corners_wide`](Self::with_sharp_corners_wide)
    /// - [`with_doubled_lines`](Self::with_doubled_lines)
    pub const fn new() -> Self {
        Self {
            margin_below: 0,
            margin_left: 1,
            branch_writer: PhantomData,
        }
    }

    /// Set the amount of margin below each annotation.
    pub const fn margin_below(&mut self, margin: usize) {
        self.margin_below = margin;
    }

    /// Set the amount of margin to the left of each annotation.
    pub const fn margin_left(&mut self, margin: usize) {
        self.margin_left = margin;
    }
}

impl Config<RoundedCorners> {
    /// Initialize with the *rounded corners* style.
    ///
    /// See the documentation for [`RoundedCorners`] for an example.
    pub const fn with_rounded_corners() -> Self {
        Self::new()
    }
}

impl Config<RoundedCornersWide> {
    /// Initialize with the *rounded corners* style and extra internal whitespace.
    ///
    /// See the documentation for [`RoundedCornersWide`] for an example.
    pub const fn with_rounded_corners_wide() -> Self {
        Self::new()
    }
}

impl Config<SharpCorners> {
    /// Initialize with the *sharp corners* style.
    ///
    /// See the documentation for [`SharpCorners`] for an example.
    pub const fn with_sharp_corners() -> Self {
        Self::new()
    }
}

impl Config<SharpCornersWide> {
    /// Initialize with the *sharp corners* style and extra internal whitespace.
    ///
    /// See the documentation for [`SharpCornersWide`] for an example.
    pub const fn with_sharp_corners_wide() -> Self {
        Self::new()
    }
}

impl Config<DoubledLines> {
    /// Initialize with the *doubled lines* style.
    ///
    /// See the documentation for [`DoubledLines`] for an example.
    pub const fn with_doubled_lines() -> Self {
        Self::new()
    }
}

/// A wrapper around an [`io::Write`] implementation which contains configuration relevant for
/// drawing branch diagrams.
///
/// Note that many small calls to `write!` are made during normal running of this program.
/// It is recommended that the output of the internal writer is buffered.
pub(crate) struct DiagramWriter<W, B> {
    /// Configuration used when drawing the branch diagram.
    writer: W,
    line_width: usize,
    queued_whitespace: usize,
    marker: PhantomData<B>,
}

impl<W: io::Write, B: WriteBranch> DiagramWriter<W, B> {
    /// Initialize a new diagram writer with the provided configuration and writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            line_width: 0,
            queued_whitespace: 0,
            marker: PhantomData,
        }
    }

    /// Returns the number of characters written since the last line break.
    pub(crate) fn line_char_count(&self) -> usize {
        self.line_width
    }

    #[inline]
    fn resolve_whitespace(&mut self, branch_width: usize) -> usize {
        if B::WIDE {
            let extra_ws = if self.line_width == 0 { 0 } else { 1 };
            let ws = extra_ws + 2 * self.queued_whitespace;
            self.line_width += branch_width + ws;

            self.queued_whitespace = 0;
            ws
        } else {
            self.line_width += self.queued_whitespace + branch_width;
            let ws = self.queued_whitespace;
            self.queued_whitespace = 0;
            ws
        }
    }

    /// Write a [`Branch`].
    #[inline]
    pub(crate) fn write_branch(&mut self, b: Branch) -> io::Result<()> {
        let ws = self.resolve_whitespace(self.branch_width(&b));

        B::write_branch(|args| self.writer.write_fmt(args), ws, b)
    }

    pub(crate) fn write_marker(&mut self, marker: char) -> io::Result<()> {
        let ws = self.resolve_whitespace(1);
        write!(&mut self.writer, "{:>ws$}{m}", "", m = marker, ws = ws)
    }

    pub(crate) fn queue_blank(&mut self, n: usize) {
        self.queued_whitespace += n;
    }

    /// Write a newline.
    #[inline]
    pub(crate) fn write_newline(&mut self) -> io::Result<()> {
        self.line_width = 0;
        self.queued_whitespace = 0;
        writeln!(&mut self.writer)
    }

    /// Write a single line of annotation, followed by a newline.
    ///
    /// The caller must guarantee the provided line does not contain any newlines.
    #[inline]
    pub(crate) fn write_annotation_line(
        &mut self,
        line: impl fmt::Display,
        bound: usize,
        padding: usize,
    ) -> io::Result<()> {
        self.queued_whitespace = 0;
        writeln!(
            &mut self.writer,
            "{:>align$}{:>padding$}{line}",
            "",
            "",
            align = bound.saturating_sub(self.line_width),
            padding = padding,
        )?;
        self.line_width = 0;
        Ok(())
    }

    fn branch_width(&self, b: &Branch) -> usize {
        if B::WIDE {
            b.width_wide()
        } else {
            b.width_narrow()
        }
    }
}

/// A special writer that a [`Generator`](crate::Generator) uses to write the characters used in
/// the branch diagram.
///
/// Implementing this trait yourself is rather annoying and should only be done in exceptional
/// situations. Usually, you just want to use a built-in implementation, such as the
/// recommended [`RoundedCorners`] style, or another style chosen from an implementation in the
/// [writer](self) module. If this is unsatisfactory, see the [`branch_writer`] macro. If this is
/// still unsatisfactory, read on!
///
/// ## Implementing [`WriteBranch`]
///
/// In order to understand how to implement [`WriteBranch`], it is important to know how a
/// [`Generator`](crate::Generator) writes a branch diagram.
///
/// Consider the following incomplete branch diagram:
/// ```txt
/// 0
/// ├┬╮
/// │1│
/// 2│╰╮
/// │╰╮│
/// ├╮││
/// 3│││
/// ╭╯││
/// ```
/// Ths branch diagram is composed of individual box-drawing characters (used for the lines), as well as the characters
/// used for the vertices (here, `0123`). It can also happen that a branch diagram has internal whitespace, in
/// which case those characters can also be part of the diagram.
///
/// The responsiblity of a [`WriteBranch`] implementation is *only* to write the lines in the
/// branch diagram.
/// In order to drive the [`WriteBranch`] implementation, the layout algorithm emits
/// [`Branch`]es, which are symbolic representations of the components used in a diagram.
/// The [`WriteBranch`] implementation takes the branch and writes the characters to the writer.
///
/// For performance reasons, instead of working directly with a [writer](io::Write), the
/// implementation is requested to generate a format template which can be immediately passed to a
/// closure for writing.
///
/// A prototypical example implementation the following.
/// ```
/// use std::{fmt, io};
/// use ramify::writer::{Branch, WriteBranch};
///
/// struct MyCustomStyle;
///
/// impl WriteBranch for MyCustomStyle {
///     const WIDE: bool = false;
///
///     fn write_branch<F>(f: F, ws: usize, b: Branch) -> io::Result<()>
///     where
///         F: for<'a> FnOnce(fmt::Arguments<'a>) -> io::Result<()> {
///         match b {
///             Branch::ForkDoubleShiftLeft(shift) => {
///                 f(format_args!("{:>ws$}╭┬{:─>shift$}╯", "", "", ws = ws, shift = shift))?;
///             }
///             _ => todo!(),
///         }
///
///         Ok(())
///     }
/// }
/// ```
/// We see that the format template does two things simultaneusly: it writes the requested whitespace at the beginning of the string,
/// and then writes the branch itself.
///
/// TODO: explain `WIDE` mode.
///
/// The width of the resulting branch (not including the whitespace prefix) must be exactly equal
/// to the result returned by [`Branch::width_narrow`] or [`Branch::width_wide`].
pub trait WriteBranch {
    /// Set this to `true` if the diagram is wide (i.e., has extra internal columns between each row),
    /// and otherwise `false.
    const WIDE: bool;

    /// Write a single branch to the provided writer, prefixed by `ws` whitespace characters.
    ///
    /// In order to optimize writes, the writer `f` only accepts an [`Arguments`](fmt::Arguments)
    /// struct, which must be generated by using the [`format_args`] macro. Repetition and other
    /// runtime-only operations must be handled with [formatting paramters](std::fmt#formatting-parameters).
    fn write_branch<F>(f: F, ws: usize, b: Branch) -> io::Result<()>
    where
        F: for<'a> FnOnce(fmt::Arguments<'a>) -> io::Result<()>;
}

branch_writer!(
    /// A style which uses rounded corners and no (unnecessary) internal whitespace.
    /// ```
    #[doc = include_str!("writer/doctest_init.txt")]
    /// # let config = Config::<RoundedCorners>::new();
    /// # let expected = "\
    /// 0
    /// ├┬╮
    /// │1├╮
    /// ││2│
    /// │3││
    /// │╭╯│
    /// ││╭┼╮
    /// │││4│
    /// ││5╭╯
    /// │6╭╯
    /// 7╭╯
    ///  8
    /// # ";
    #[doc = include_str!("writer/doctest_end.txt")]
    /// ```
    pub struct RoundedCorners {
        charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
        wide: false
    }
);

branch_writer!(
    /// A style which uses sharp corners and no (unnecessary) internal whitespace.
    /// ```
    #[doc = include_str!("writer/doctest_init.txt")]
    /// # let config = Config::<SharpCorners>::new();
    /// # let expected = "\
    /// 0
    /// ├┬┐
    /// │1├┐
    /// ││2│
    /// │3││
    /// │┌┘│
    /// ││┌┼┐
    /// │││4│
    /// ││5┌┘
    /// │6┌┘
    /// 7┌┘
    ///  8
    /// # ";
    #[doc = include_str!("writer/doctest_end.txt")]
    /// ```
    pub struct SharpCorners {
        charset: ["│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"],
        wide: false
    }
);

branch_writer!(
    /// A style which uses rounded corners and additional internal whitespace.
    /// ```
    #[doc = include_str!("writer/doctest_init.txt")]
    /// # let config = Config::<RoundedCornersWide>::new();
    /// # let expected = "\
    /// 0
    /// ├─┬─╮
    /// │ 1 ├─╮
    /// │ │ 2 │
    /// │ 3 │ │
    /// │ ╭─╯ │
    /// │ │ ╭─┼─╮
    /// │ │ │ 4 │
    /// │ │ 5 ╭─╯
    /// │ 6 ╭─╯
    /// 7 ╭─╯
    ///   8
    /// # ";
    #[doc = include_str!("writer/doctest_end.txt")]
    /// ```
    pub struct RoundedCornersWide {
        charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
        wide: true,
    }
);

branch_writer!(
    /// A style which uses sharp corners and additional internal whitespace.
    /// ```
    #[doc = include_str!("writer/doctest_init.txt")]
    /// # let config = Config::<SharpCornersWide>::new();
    /// # let expected = "\
    /// 0
    /// ├─┬─┐
    /// │ 1 ├─┐
    /// │ │ 2 │
    /// │ 3 │ │
    /// │ ┌─┘ │
    /// │ │ ┌─┼─┐
    /// │ │ │ 4 │
    /// │ │ 5 ┌─┘
    /// │ 6 ┌─┘
    /// 7 ┌─┘
    ///   8
    /// # ";
    #[doc = include_str!("writer/doctest_end.txt")]
    /// ```
    pub struct SharpCornersWide {
        charset: ["│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"],
        wide: true
    }
);

branch_writer!(
    /// A style which uses doubled lines and additional internal whitespace.
    /// ```
    #[doc = include_str!("writer/doctest_init.txt")]
    /// # let config = Config::<DoubledLines>::new();
    /// # let expected = "\
    /// 0
    /// ╠═╦═╗
    /// ║ 1 ╠═╗
    /// ║ ║ 2 ║
    /// ║ 3 ║ ║
    /// ║ ╔═╝ ║
    /// ║ ║ ╔═╬═╗
    /// ║ ║ ║ 4 ║
    /// ║ ║ 5 ╔═╝
    /// ║ 6 ╔═╝
    /// 7 ╔═╝
    ///   8
    /// # ";
    #[doc = include_str!("writer/doctest_end.txt")]
    /// ```
    pub struct DoubledLines {
        charset: ["║", "═", "╗", "╔", "╝", "╚", "╣", "╠", "╦", "╬"],
        wide: true
    }
);
