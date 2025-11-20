/// A single component used when drawing a branch diagram.
///
/// See the documentation for [`WriteBranch`](crate::writer::WriteBranch) for more detail on when
/// this struct is expected.
#[derive(Debug, Clone, Copy)]
pub enum Branch {
    /// A vertex marker character.
    Marker(char),
    /// A `╭╯` left shift.
    ///
    /// The field is the number of extra horizontal spacers.
    ShiftLeft(usize),
    /// A `│` continuation.
    Continue,
    /// A `╰╮` right shift.
    ///
    /// The field is the number of extra horizontal spacers. For example, `ShiftRight(2)` is `╰──╮`.
    ShiftRight(usize),
    /// A `╭┬╯` left shift and double fork.
    ///
    /// The field is the number of extra horizontal spacers. For example, `ForkDoubleShiftLeft(1)`
    /// is ╭┬─╯
    ForkDoubleShiftLeft(usize),
    /// A `╭┤` left double fork.
    ForkDoubleLeft,
    /// A `├╮` right double fork.
    ForkDoubleRight,
    /// A `╰┬╮` right shift and double fork
    ///
    /// The field is the number of extra horizontal spacers. For example, `ForkDoubleShiftRight(2)`
    /// is `╭┬┬──╯`.
    ForkDoubleShiftRight(usize),
    /// A `╭┬┬╯` left shift and triple fork.
    ///
    /// The field is the number of extra horizontal spacers. For example, `ForkTripleShiftLeft(1)`
    /// is `╭┬┬─╯`.
    ForkTripleShiftLeft(usize),
    /// A `╭┬┤` left triple fork.
    ForkTripleLeft,
    /// A `╭┼╮` middle triple fork.
    ForkTripleMiddle,
    /// A `├┬╮` right triple fork.
    ForkTripleRight,
    /// A `╰┬┬╮` right shift and triple fork.
    ///
    /// The field is the number of extra horizontal spacers. For example,
    /// `ForkTripleShiftRight(1)` is `╰─┬┬╮`.
    ForkTripleShiftRight(usize),
}

impl Branch {
    /// The number of characters that a [`Branch`] occupies in the branch diagram with a given
    /// gutter width. The implementation is guaranteed to be exactly as follows:
    /// ```
    /// use ramify::writer::Branch;
    /// fn width(b: &Branch, gutter_width: usize) -> usize {
    ///     let base_width = match b {
    ///         Branch::Marker(_) | Branch::Continue => 1,
    ///         Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 2 + shift,
    ///         Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 2,
    ///         Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => 3 + shift,
    ///         Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => 4 + shift,
    ///         Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 3,
    ///     };
    ///     base_width + (base_width - 1) * gutter_width
    /// }
    /// # let b = Branch::ForkTripleShiftRight(12);
    /// # assert_eq!(b.width(1), 31);
    /// # assert_eq!(width(&b, 1), 31);
    /// ```
    pub fn width(&self, gutter_width: usize) -> usize {
        let base_width = match self {
            Branch::Continue | Branch::Marker(_) => 1,
            Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 2 + shift,
            Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 2,
            Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => 3 + shift,
            Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => 4 + shift,
            Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 3,
        };
        base_width + (base_width - 1) * gutter_width
    }
}

/// A macro to generate a custom branch diagram style from a list of box-drawing characters.
///
/// The macro expects a struct with standard visiblilty parameters,
/// with custom struct-like syntax to specify the list of characters to use in the branch diagram and the amount of internal whitespace.
///
/// For example, to implement a mix of [`SharpCornersWide`](crate::writer::SharpCornersWide) and [`RoundedCornersWide`](crate::writer::RoundedCornersWide) with
/// local visiblity and extra internal whitespace:
/// ```
/// use ramify::writer::branch_writer;
///
/// branch_writer!(
///     /// A style which mixes rounded corners and sharp corners, with a lot of
///     /// internal whitespace.
///     pub(crate) struct MixedCornersExtraWide {
///         charset: ["│", "─", "┐", "┌", "╯", "╰", "┤", "├", "┬", "┼"],
///         gutter_width: 2, // {Rounded/Sharp}CornersWide uses `gutter_width = 1`
///     }
/// );
/// ```
/// The resulting struct will be a unit struct `pub(crate) struct MixedCornersExtraWide;` which implements the
/// [`WriteBranch`](crate::writer::WriteBranch) trait. Any struct attributes (such as docstrings or derives) are propagated.
///
/// The order in the `charset` field must match the order above. The string literals in the
/// `charset` field should be single characters which have width 1 when printed to the terminal, or
/// the resulting branch diagram will be corrupted. A good choice is to use [box-drawing
/// characters](https://en.wikipedia.org/wiki/Box-drawing_characters).
/// See the [`Branch`] struct for more detail on how the characters are expected to be used.
///
/// The `gutter_width` field is the number of extra unused columns placed between the vertical
/// lines.
#[macro_export]
macro_rules! branch_writer{
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal$(,)?],
            gutter_width: $gutter_width:expr$(,)?
        }
    ) => {
        $(#[$outer])*
        $vis struct $name;

        impl $crate::writer::WriteBranch for $name {
            const GUTTER_WIDTH: usize = $gutter_width;

            fn write_branch<F>(f: F, ws: usize, b: $crate::writer::Branch) -> ::std::io::Result<()>
            where
                F: for<'a> FnOnce(::std::fmt::Arguments<'a>) -> ::std::io::Result<()>,
            {
                match b {
                    $crate::writer::Branch::Marker(m) => {
                        f(::std::format_args!("{:>ws$}{m}", "", ws = ws))
                    }
                    $crate::writer::Branch::ShiftLeft(shift) => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $se, "{:", $ew, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }
                    $crate::writer::Branch::Continue => f(::std::format_args!(concat!("{:>ws$}", $ns), "", ws = ws)),
                    $crate::writer::Branch::ShiftRight(shift) => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $ne, "{:", $ew, ">shift$}", $sw),
                            "",
                            "",
                            ws = ws,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }

                    $crate::writer::Branch::ForkDoubleShiftLeft(shift) => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $se, "{:", $ew, ">gutter$}" , $sew, "{:", $ew, ">shift$}", $nw),
                            "",
                            "",
                            "",
                            ws = ws,
                            gutter = $gutter_width,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }
                    $crate::writer::Branch::ForkDoubleLeft => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $se, "{:", $ew, ">gutter$}", $nsw),
                            "",
                            "",
                            gutter = $gutter_width,
                            ws = ws
                        ))
                    }
                    $crate::writer::Branch::ForkDoubleRight => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $nse, "{:", $ew, ">gutter$}", $sw),
                            "",
                            "",
                            gutter = $gutter_width,
                            ws = ws
                        ))
                    }
                    $crate::writer::Branch::ForkDoubleShiftRight(shift) => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $nw, "{:", $ew, ">shift$}", $sew, "{:", $ew, ">gutter$}", $sw),
                            "",
                            "",
                            "",
                            ws = ws,
                            gutter = $gutter_width,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }

                    $crate::writer::Branch::ForkTripleShiftLeft(shift) => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $se, "{:", $ew, ">gutterl$}", $sew, "{:", $ew, ">gutterr$}", $sew, "{:", $ew, ">shift$}", $nw),
                            "",
                            "",
                            "",
                            "",
                            gutterl = $gutter_width,
                            gutterr= $gutter_width,
                            ws = ws,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }
                    $crate::writer::Branch::ForkTripleLeft => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $se, "{:", $ew, ">gutterl$}", $sew, "{:", $ew, ">gutterr$}", $nsw),
                            "",
                            "",
                            "",
                            gutterl = $gutter_width,
                            gutterr = $gutter_width,
                            ws = ws
                        ))
                    }
                    $crate::writer::Branch::ForkTripleMiddle => {
                        f(::std::format_args!(concat!("{:>ws$}", $se, "{:", $ew, ">gutterl$}", $nsew, "{:", $ew, ">gutterr$}", $sw), "", "", "", gutterl = $gutter_width, gutterr = $gutter_width, ws = ws))
                    }
                    $crate::writer::Branch::ForkTripleRight => {
                        f(::std::format_args!(concat!("{:>ws$}", $nse, "{:", $ew, ">gutterl$}", $sew, "{:", $ew, ">gutterr$}", $sw), "", "", "", gutterl = $gutter_width, gutterr = $gutter_width, ws = ws))
                    }
                    $crate::writer::Branch::ForkTripleShiftRight(shift) => {
                        f(::std::format_args!(
                            concat!("{:>ws$}", $ne, "{:", $ew, ">shift$}", $sew, "{:", $ew, ">gutterl$}", $sew, "{:", $ew, ">gutterr$}", $sw),
                            "",
                            "",
                            "",
                            "",
                            ws = ws,
                            gutterl = $gutter_width,
                            gutterr = $gutter_width,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }
                }
            }
        }
    };
}

pub use branch_writer;
