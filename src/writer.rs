//! # Configuration and styles for writing the branch diagram writer.
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
//! Here is a gallery of the default line styles. Here are the links:
//!
//! > [`RoundedCorners`]; [`SharpCorners`]; [`RoundedCornersWide`]; [`SharpCornersWide`]; [`DoubledLines`]
//!
//! ```txt
//! rounded  sharp    rounded     sharp       doubled
//! corners  corners  corners     corners     lines
//!                   wide        wide
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
//! This section will be written some time in the future!
mod branch;

pub use self::branch::{Branch, branch_writer};

use std::{fmt, io, marker::PhantomData};

/// Configuration passed to a [`Generator`](crate::Generator) to control the appearance
/// and layout of the branch diagram and associated annotations.
///
/// Configuration is separated into *layout* and *style*.
///
/// Layout is configured dynamically.
/// See the public struct field documentation for a short description of the configuration parameters.
/// This configuration can be modified while a generator is running using
/// [`Generator::config_mut`](crate::Generator::config_mut).
///
/// The associated type parameter `B` is the diagram style which should implement [`WriteBranch`]. This includes features like whether to
/// use sharp or rounded corners or the amount of extra internal spacing (gutters). This must be
/// specified at compilation time. The built-in styles are documented in the [module-level
/// docs](self#style-gallery).
///
/// Note that the width numbers may be in terms of gutters rather than characters. If the gutter width
/// is 0, this is the the same as the character width. In general, if the width is `n`, the
/// resulting number of characters is `(gutter_width + 1) * n`.
#[derive(Debug, Clone)]
pub struct Config<B = RoundedCorners> {
    /// Extra padding before each vertex. The padding applies after the annotation, or after the
    /// vertex if there is no annotation. This is the number of characters. The default is `0`.
    pub row_padding: usize,
    /// The margin between the annotation and the branch diagram. This is the number of characters. The default is `1`.
    pub annotation_margin: usize,
    /// Whether or not to allow extra an extra column of width slack which can result in slightly
    /// shorter branch diagrams. This is at the cost of pushing the annotation to the right
    /// unnecessarily by the gutter width if the extra slack is not used. The default is `false`.
    pub width_slack: bool,
    /// The minimum width of the diagram. Annotations will never begin earlier than this.
    /// Margin requested in `margin_left` is added to of this parameter. This is the number of
    /// gutters. The default value is `0`.
    pub min_diagram_width: usize,
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
            row_padding: 0,
            annotation_margin: 1,
            width_slack: false,
            min_diagram_width: 0,
            branch_writer: PhantomData,
        }
    }

    pub(crate) fn normalize_diagram_width(&self, width: usize) -> usize {
        let slack: usize = self.width_slack.into();
        (width + slack).max(self.min_diagram_width)
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
        let extra_ws = if self.line_width == 0 {
            0
        } else {
            B::GUTTER_WIDTH
        };
        let ws = extra_ws + (1 + B::GUTTER_WIDTH) * self.queued_whitespace;
        self.line_width += branch_width + ws;

        self.queued_whitespace = 0;
        ws
    }

    /// Write a [`Branch`].
    #[inline]
    pub(crate) fn write_branch(&mut self, b: Branch) -> io::Result<()> {
        let ws = self.resolve_whitespace(self.branch_width(&b));

        B::write_branch(|args| self.writer.write_fmt(args), ws, b)
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
        b.width(B::GUTTER_WIDTH)
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
/// [`Generator`](crate::Generator) writes a branch diagram. We will refer to a [`WriteBranch`]
/// implementation as a *branch writer*.
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
/// The responsiblity of a branch writer is write the individual components of the branch diagram.
/// However, a branch writer knows nothing about the current state: the state itself is held by
/// the [`Generator`](crate::Generator) which then requests the relevant text from the branch
/// writer. These requests take the form of [`Branch`]es, which represent the components used in the diagram. The branch writer satisfies a request in the [`write_branch`](WriteBranch::write_branch) function.
///
/// For performance reasons, instead of working directly with a [writer](io::Write), the
/// implementation is requested to generate a format template which can be immediately passed to a
/// closure for writing.
///
/// ### Basic example
///
/// A prototypical example implementation the following.
/// ```
/// use std::{fmt, io};
/// use ramify::writer::{Branch, WriteBranch};
///
/// struct MyCustomStyle;
///
/// impl WriteBranch for MyCustomStyle {
///     const GUTTER_WIDTH: usize = 0;
///
///     fn write_branch<F>(f: F, ws: usize, b: Branch) -> io::Result<()>
///     where
///         F: for<'a> FnOnce(fmt::Arguments<'a>) -> io::Result<()> {
///         match b {
///             Branch::ForkDoubleShiftLeft(shift) => {
///                 f(format_args!("{:>ws$}╭┬{:─>shift$}╯",
///                     "",
///                     "",
///                     ws = ws,
///                     shift = shift
///                 ))
///             }
///             _ => todo!(),
///         }
///     }
/// }
/// ```
/// We see that the format template does two things simultaneusly: it writes the requested whitespace at the beginning of the string,
/// and then writes the branch itself.
///
/// ### The expected width of the branch
///
/// Since the branch writer and the [`Generator`](crate::Generator) must be able to write different
/// parts of the tree together, they must agree on how many characters a given write operation will
/// occupy.
///
/// Width computations are important for many purposes. For example, correct alignment of
/// annotations requires the [`Generator`](crate::Generator) to keep track of the number of
/// characters which have been written in the line so far, and also to know that subsequent lines
/// will not draw so many additional characters that they will overlap with the annotation.
///
/// The width parameter is controlled by the associated [`GUTTER_WIDTH`](WriteBranch::GUTTER_WIDTH)
/// parameter. This is the number of spaces between vertical branches. For example:
/// ```txt
/// width 0  width 1  width 2
///
/// 0        0        0
/// ├┬╮      ├─┬─╮    ├──┬──╮
/// │1│      │ 1 │    │  1  │
/// 2│╰╮     2 │ ╰─╮  2  │  ╰──╮
///           ^ ^ ^    ^^ ^^ ^^
/// ```
/// The branch writer is only responsible for writing the number of characters internal to the [`Branch`]
/// that it is writing. The correct number of preceding spaces is passed in the `ws` parameter.
/// For example, if `GUTTER_WIDTH = 0`, a [`Branch::ForkDoubleShiftLeft`] with field `1` is written
/// like `╭┬─╯`. However, if `GUTTER_WIDTH = 1`, then it is written like `╭─┬───╯`.
///
/// The exact number of expected characters is documented in [`Branch::width`].
///
/// ### Example with non-zero gutter width
/// Here, the gutter width is 2. Note that we need to add extra horizontal `─` components in two
/// places: between the down forks (i.e. `╭┬`), and also between beteen the requested horizontal
/// spacers in `shift`.
/// ```
/// use std::{fmt, io};
/// use ramify::writer::{Branch, WriteBranch};
///
/// struct MyCustomStyle;
///
/// impl WriteBranch for MyCustomStyle {
///     const GUTTER_WIDTH: usize = 2;
///
///     fn write_branch<F>(f: F, ws: usize, b: Branch) -> io::Result<()>
///     where
///         F: for<'a> FnOnce(fmt::Arguments<'a>) -> io::Result<()> {
///         match b {
///             Branch::ForkDoubleShiftLeft(shift) => {
///                 f(format_args!("{:>ws$}╭──┬{:─>shift$}╯",
///                     "",
///                     "",
///                     ws = ws,
///                     shift = 3 * shift + 2
///                 ))
///             }
///             _ => todo!(),
///         }
///     }
/// }
/// ```
pub trait WriteBranch {
    /// The number of extra internal columns.
    const GUTTER_WIDTH: usize;

    /// Whether or not to generate the tree in 'inverted' mode.
    const INVERTED: bool = false;

    /// Write a single branch to the provided writer, prefixed by `ws` whitespace characters.
    ///
    /// In order to optimize writes, the writer `f` only accepts an [`Arguments`](fmt::Arguments)
    /// struct, which must be generated by using the [`format_args!`] macro. Repetition and other
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
    /// This is generated using the [`branch_writer!`] macro as follows:
    /// ```
    /// # use ramify::writer::branch_writer;
    /// branch_writer! {
    ///     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    ///     pub struct RoundedCorners {
    ///         charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
    ///         gutter_width: 0,
    ///     }
    /// }
    /// ```
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct RoundedCorners {
        charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
        gutter_width: 0,
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
    /// This is generated using the [`branch_writer!`] macro as follows:
    /// ```
    /// # use ramify::writer::branch_writer;
    /// branch_writer! {
    ///     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    ///     pub struct SharpCorners {
    ///         charset: ["│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"],
    ///         gutter_width: 0,
    ///     }
    /// }
    /// ```
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct SharpCorners {
        charset: ["│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"],
        gutter_width: 0,
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
    /// This is generated using the [`branch_writer!`] macro as follows:
    /// ```
    /// # use ramify::writer::branch_writer;
    /// branch_writer! {
    ///     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    ///     pub struct RoundedCornersWide {
    ///         charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
    ///         gutter_width: 1,
    ///     }
    /// }
    /// ```
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct RoundedCornersWide {
        charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
        gutter_width: 1,
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
    /// This is generated using the [`branch_writer!`] macro as follows:
    /// ```
    /// # use ramify::writer::branch_writer;
    /// branch_writer! {
    ///     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    ///     pub struct SharpCornersWide {
    ///         charset: ["│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"],
    ///         gutter_width: 1,
    ///     }
    /// }
    /// ```
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct SharpCornersWide {
        charset: ["│", "─", "┐", "┌", "┘", "└", "┤", "├", "┬", "┼"],
        gutter_width: 1,
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
    /// This is generated using the [`branch_writer!`] macro as follows:
    /// ```
    /// # use ramify::writer::branch_writer;
    /// branch_writer! {
    ///     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    ///     pub struct DoubledLines {
    ///         charset: ["║", "═", "╗", "╔", "╝", "╚", "╣", "╠", "╦", "╬"],
    ///         gutter_width: 1,
    ///     }
    /// }
    /// ```
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct DoubledLines {
        charset: ["║", "═", "╗", "╔", "╝", "╚", "╣", "╠", "╦", "╬"],
        gutter_width: 1,
    }
);
