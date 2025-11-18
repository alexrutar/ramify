//! # Configuration and styles for the writer
//!
//! This module contains the [`Config`] struct, which defines the configuration used by the
//! [`Generator`](crate::Generator) when drawing the branch diagram.
//!
//! The [`Config`] struct uses compile-time specification of the drawing style. You can implement
//! your own; see the [`BranchWrite`] trait. An implementation of this trait in simple cases can be
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

mod branch;

use self::branch::branch_writer_impl;
pub use self::branch::{Branch, branch_writer};

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

impl<'a, W: io::Write, B: BranchWrite> DiagramWriter<W, B> {
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
            self.branch_width_wide(b)
        } else {
            self.branch_width_narrow(b)
        }
    }

    fn branch_width_narrow(&self, b: &Branch) -> usize {
        match b {
            Branch::Continue => 1,
            Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 2 + shift,
            Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 2,
            Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => 3 + shift,
            Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => 4 + shift,
            Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 3,
        }
    }

    fn branch_width_wide(&self, b: &Branch) -> usize {
        match b {
            Branch::Continue => 1,
            Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 3 + 2 * shift,
            Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 3,
            Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => {
                5 + 2 * shift
            }
            Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => {
                7 + 2 * shift
            }
            Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 5,
        }
    }
}

/// TODO
pub trait BranchWrite {
    /// TODO
    const WIDE: bool;

    /// TODO
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
    pub struct RoundedCorners;

    charset => {"│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"},
    wide => false
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
    pub struct SharpCorners;

    charset => {"│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"},
    wide => false
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
    pub struct RoundedCornersWide;

    charset => {"│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"},
    wide => true
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
    pub struct SharpCornersWide;

    charset => {"│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"},
    wide => true
);

branch_writer!(
    /// A style which uses doubled lines corners and additional internal whitespace.
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
    pub struct DoubledLines;

    charset => {"║", "═", "╗", "╔", "╝", "╚", "╣", "╠", "╦", "╬"},
    wide => true
);
