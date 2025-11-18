/// A single component used when drawing a branch diagram.
///
/// See the documentation for [`WriteBranch`](crate::writer::WriteBranch) for more detail on when
/// this struct is expected.
#[derive(Debug, Clone, Copy)]
pub enum Branch {
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
    /// The number of characters that a [`Branch`] occupies in the branch diagram in 'narrow'
    /// writing mode. The implementation is guaranteed to be exactly as follows:
    /// ```
    /// use ramify::writer::Branch;
    /// fn width_narrow(b: &Branch) -> usize {
    ///     match b {
    ///         Branch::Continue => 1,
    ///         Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 2 + shift,
    ///         Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 2,
    ///         Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => 3 + shift,
    ///         Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => 4 + shift,
    ///         Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 3,
    ///     }
    /// }
    /// # let b = Branch::ForkDoubleShiftLeft(12);
    /// # assert_eq!(b.width_narrow(), 15);
    /// # assert_eq!(width_narrow(&b), 15);
    /// ```
    pub fn width_narrow(&self) -> usize {
        match self {
            Branch::Continue => 1,
            Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 2 + shift,
            Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 2,
            Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => 3 + shift,
            Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => 4 + shift,
            Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 3,
        }
    }

    /// The number of characters that a [`Branch`] occupies in the branch diagram in 'wide'
    /// writing mode. The implementation is exactly as follows:
    /// ```
    /// use ramify::writer::Branch;
    /// fn width_wide(b: &Branch) -> usize {
    ///     match b {
    ///         Branch::Continue => 1,
    ///         Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 3 + 2 * shift,
    ///         Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 3,
    ///         Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => {
    ///             5 + 2 * shift
    ///         }
    ///         Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => {
    ///             7 + 2 * shift
    ///         }
    ///         Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 5,
    ///     }
    /// }
    /// # let b = Branch::ForkTripleShiftRight(12);
    /// # assert_eq!(b.width_wide(), 31);
    /// # assert_eq!(width_wide(&b), 31);
    /// ```
    pub fn width_wide(&self) -> usize {
        match self {
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

/// This is an implementation detail for the [`branch_writer`] macro and should not be imported!
#[macro_export]
#[doc(hidden)]
macro_rules! __branch_writer_impl {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident;

        chars => {$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal},
        align => {$wide:expr, $mul:expr, $pos:expr},
        shift => $shift:expr
    ) => {
        $(#[$outer])*
        $vis struct $name;

        impl $crate::writer::WriteBranch for $name {
            const WIDE: bool = $wide;

            fn write_branch<F>(f: F, ws: usize, b: $crate::writer::Branch) -> ::std::io::Result<()>
            where
                F: for<'a> FnOnce(::std::fmt::Arguments<'a>) -> ::std::io::Result<()>,
            {
                let args = match b {
                    $crate::writer::Branch::ShiftLeft(shift) => {
                        ::std::format_args!(
                            concat!("{:>ws$}", $se, "{:", $shift, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                    $crate::writer::Branch::Continue => ::std::format_args!(concat!("{:>ws$}", $ns), "", ws = ws),
                    $crate::writer::Branch::ShiftRight(shift) => {
                        ::std::format_args!(
                            concat!("{:>ws$}", $ne, "{:", $shift, ">shift$}", $sw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }

                    $crate::writer::Branch::ForkDoubleShiftLeft(shift) => {
                        ::std::format_args!(
                            concat!("{:>ws$}", $se, $ew, $sew, "{:", $shift, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                    $crate::writer::Branch::ForkDoubleLeft => {
                        ::std::format_args!(concat!("{:>ws$}", $se, $ew, $nsw), "", ws = ws)
                    }
                    $crate::writer::Branch::ForkDoubleRight => {
                        ::std::format_args!(concat!("{:>ws$}", $nse, $ew, $sw), "", ws = ws)
                    }
                    $crate::writer::Branch::ForkDoubleShiftRight(shift) => {
                        ::std::format_args!(
                            concat!("{:>ws$}", $nw, "{:", $shift, ">shift$}", $sew, $ew, $sw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }

                    $crate::writer::Branch::ForkTripleShiftLeft(shift) => {
                        ::std::format_args!(
                            concat!("{:>ws$}", $se, $ew, $sew, $ew, $sew, "{:", $shift, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                    $crate::writer::Branch::ForkTripleLeft => {
                        ::std::format_args!(concat!("{:>ws$}", $se, $ew, $sew, $ew, $nsw), "", ws = ws)
                    }
                    $crate::writer::Branch::ForkTripleMiddle => {
                        ::std::format_args!(concat!("{:>ws$}", $se, $ew, $nsew, $ew, $sw), "", ws = ws)
                    }
                    $crate::writer::Branch::ForkTripleRight => {
                        ::std::format_args!(concat!("{:>ws$}", $nse, $ew, $sew, $ew, $sw), "", ws = ws)
                    }
                    $crate::writer::Branch::ForkTripleShiftRight(shift) => {
                        ::std::format_args!(
                            concat!("{:>ws$}", $ne, "{:", $shift, ">shift$}", $sew, $ew, $sew, $ew, $sw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                };
                f(args)
            }
        }
    };
}

/// A macro to generate a [`WriteBranch`](crate::writer::WriteBranch) implementation from a
/// list of box-drawing characters.
///
/// The macro expects a struct with standard visiblilty parameters,
/// followed by a custom internal syntax to specify the list of characters to use in the branch diagram and whether or not to include
/// internal whitespace.
///
/// For example, to implement [`RoundedCornersWide`](crate::writer::RoundedCornersWide) with
/// local visiblity:
/// ```
/// use ramify::writer::branch_writer;
///
/// branch_writer!(
///     /// A style which uses rounded corners and additional internal whitespace.
///     pub(crate) struct RoundedCornersWide {
///         charset: ["│", "─", "╮", "╭", "╯", "╰", "┤", "├", "┬", "┼"],
///         wide: true,
///     }
/// );
/// ```
/// The resulting struct will be a unit struct `pub(crate) struct RoundedCornersWide;` which implements the
/// [`WriteBranch`](crate::writer::WriteBranch) trait. Any struct attributes (such as docstrings or derives) are propagated.
///
/// The order in the `charset` field must match the order above. The string literals in the
/// `charset` field should be single characters which have width 1 when printed to the terminal, or
/// the resulting branch diagram will be corrupted. A good choice is to use [box-drawing
/// characters](https://en.wikipedia.org/wiki/Box-drawing_characters).
///
/// See the [`Branch`] struct for more detail on how the characters are expected to be used.
#[macro_export]
macro_rules! branch_writer {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal$(,)?],
            wide: false$(,)?
        }
    ) => {
        $crate::writer::__branch_writer_impl!(
            $(#[$outer])*
            $vis struct $name;

            chars => {$ns, "", $sw, $se, $nw, $ne, $nsw, $nse, $sew, $nsew},
            align => {false, 1, 0},
            shift => $ew
        );
    };
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal$(,)?],
            wide: true$(,)?
        }
    ) => {
        $crate::writer::__branch_writer_impl!(
            $(#[$outer])*
            $vis struct $name;

            chars => {$ns, $ew, $sw, $se, $nw, $ne, $nsw, $nse, $sew, $nsew},
            align => {true, 2, 1},
            shift => $ew
        );
    };
}

pub use __branch_writer_impl;
pub use branch_writer;
