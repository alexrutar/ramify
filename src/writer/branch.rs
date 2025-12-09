/// A single component used when drawing a branch diagram.
///
/// See the documentation for [`WriteBranch`](crate::writer::WriteBranch) for more detail on when
/// this struct is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    /// A vertex marker character.
    Marker(char),
    /// A `│` continuation.
    Continue,
    /// A `╭┬─╯` left shift.
    ///
    /// The first field is the number of horizontal spacers and the second field
    /// is the number of new branches.
    ShiftForkLeft(usize, usize),
    /// A `╰─┬╮` right shift and fork.
    ///
    /// The first field is the number of `─` horizontal spacers and the second field
    /// is the number of `┬` new branches.
    ShiftForkRight(usize, usize),
    /// A `╭┬┤` left fork.
    ///
    /// The field is the number of `┬` forks.
    ForkLeft(usize),
    /// A `├┬╮` right fork.
    ///
    /// The field is the number of `┬` forks.
    ForkRight(usize),
    /// A `╭┬┼┬╮` middle fork.
    ///
    /// The first field is the number of `┬` forks on the left, and the second is the
    /// number of `┬` forks on the right.
    ForkMiddle(usize, usize),
    /// A `╯` left merge starter.
    MergeLeft,
    /// A `┴` central merge joiner.
    MergeCenter,
    /// A `╰` right merge starter.
    MergeRight,
    /// A `│` or `─` traverse.
    ///
    /// This is a horizontal line crossing a vertical line.
    Traverse,
}

impl Branch {
    /// The number of characters that a [`Branch`] occupies in the branch diagram with a given
    /// gutter width. The implementation is exactly as follows:
    /// ```
    /// use ramify::writer::Branch;
    /// fn width(b: &Branch, gutter_width: usize) -> usize {
    ///     let base_width = match b {
    ///          Branch::Continue
    ///          | Branch::Marker(_)
    ///          | Branch::MergeLeft
    ///          | Branch::MergeCenter
    ///          | Branch::MergeRight
    ///          | Branch::Traverse => 1,
    ///         Branch::ShiftForkLeft(shift, fork) | Branch::ShiftForkRight(shift, fork) => 2 + shift + fork,
    ///         Branch::ForkLeft(fork) | Branch::ForkRight(fork) => 2 + fork,
    ///         Branch::ForkMiddle(l, r) => 3 + l + r,
    ///     };
    ///     base_width + (base_width - 1) * gutter_width
    /// }
    /// # let b = Branch::ShiftForkRight(12, 2);
    /// # assert_eq!(b.width(1), 31);
    /// # assert_eq!(width(&b, 1), 31);
    /// ```
    pub const fn width(&self, gutter_width: usize) -> usize {
        let base_width = match self {
            Branch::Continue
            | Branch::Marker(_)
            | Branch::MergeLeft
            | Branch::MergeCenter
            | Branch::MergeRight
            | Branch::Traverse => 1,
            Branch::ShiftForkLeft(shift, fork) | Branch::ShiftForkRight(shift, fork) => {
                shift.wrapping_add(*fork).wrapping_add(2)
            }
            Branch::ForkLeft(fork) | Branch::ForkRight(fork) => fork.wrapping_add(2),
            Branch::ForkMiddle(l, r) => l.wrapping_add(*r).wrapping_add(3),
        };
        base_width + (base_width - 1) * gutter_width
    }

    /// A convenience function to move the left hand alignment by a given amount and then perform
    /// the fork.
    ///
    /// The first argument is the relative left alignment: how many characters to the left the
    /// entire group should move. The second argument is the number of new branches.
    ///
    /// To align right, use [`Branch::ShiftForkRight`].
    ///
    /// # Examples
    /// ```
    /// use ramify::writer::Branch;
    ///
    /// // ╭┬┼┬┬╮
    /// assert_eq!(Branch::align_left_and_fork(2, 5), Branch::ForkMiddle(1, 2));
    ///
    /// // │
    /// assert_eq!(Branch::align_left_and_fork(0, 0), Branch::Continue);
    /// // ╭┬┬┤
    /// assert_eq!(Branch::align_left_and_fork(3, 3), Branch::ForkLeft(2));
    ///
    /// // ╭┬─╯
    /// assert_eq!(Branch::align_left_and_fork(3, 1), Branch::ShiftForkLeft(1, 1));
    ///
    /// // ├╮
    /// assert_eq!(Branch::align_left_and_fork(0, 1), Branch::ForkRight(0));
    /// ```
    pub const fn align_left_and_fork(align: usize, forks: usize) -> Self {
        if align == 0 {
            match forks.checked_sub(1) {
                None => Branch::Continue,
                Some(n) => Branch::ForkRight(n),
            }
        } else if align < forks {
            Branch::ForkMiddle(align - 1, forks - align - 1)
        } else if align == forks {
            // forks > 0 since align > 0
            Branch::ForkLeft(forks - 1)
        } else {
            // align > forks
            Branch::ShiftForkLeft(align - forks - 1, forks)
        }
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
///         charset: ["│", "─", "┐", "┌", "╯", "╰", "┤", "├", "┬", "┴", "┼"],
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
///
/// ## Inverting annotations
///
/// An optional `inverted` field can be set to true, which will cause annotations to be written in
/// reverse order, with the vertex marker being written on the last row of annotation.
/// This makes the annotations look correct if the lines of the branch diagram are printed in
/// reverse order. Doing this also requires inverting the caracter set, as shown in the below example.
/// ```
/// use ramify::writer::branch_writer;
///
/// branch_writer!(
///     /// An inverted style.
///     struct Inverted {
///         charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┬", "┼"],
///         // inverted chars:   ^    ^     ^    ^              ^    ^
///         gutter_width: 0,
///         inverted: true,
///     }
/// );
/// ```
/// A complete example can be found in the [examples
/// folder](https://github.com/alexrutar/ramify/tree/master/examples).
#[macro_export]
macro_rules! branch_writer {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $new:literal, $nsew:literal$(,)?],
            gutter_width: $gutter_width:expr,
            inverted: $inverted:expr$(,)?
        }
    ) => {
        $(#[$outer])*
        $vis struct $name;

        impl $crate::writer::WriteBranch for $name {
            const GUTTER_WIDTH: usize = $gutter_width;

            const INVERTED: bool = $inverted;

            fn write_branch<F>(mut f: F, ws: usize, b: $crate::writer::Branch) -> ::std::io::Result<()>
            where
                F: for<'a> FnMut(::std::fmt::Arguments<'a>) -> ::std::io::Result<()>,
            {
                // FIXME: specialize for the following cases?
                // - fork = 0
                // - fork = 1
                // - fork = 2,
                // - gutter_width = 0
                match b {
                    $crate::writer::Branch::Marker(m) => {
                        f(::std::format_args!("{:>ws$}{m}", "", ws = ws))
                    }
                    $crate::writer::Branch::ShiftForkLeft(shift, fork) => {
                        f(::std::format_args!(::std::concat!("{:>ws$}", $se), "", ws = ws))?;
                        for _ in 0..fork {
                            f(::std::format_args!(::std::concat!("{:", $ew, ">w$}", $sew), "", w = $gutter_width))?;
                        }
                        f(::std::format_args!(
                                ::std::concat!("{:", $ew, ">shift$}", $nw),
                                "",
                                shift = ($gutter_width + 1) * shift + $gutter_width
                        ))
                    }
                    $crate::writer::Branch::Continue | $crate::writer::Branch::Traverse => f(::std::format_args!(::std::concat!("{:>ws$}", $ns), "", ws = ws)),
                    $crate::writer::Branch::ShiftForkRight(shift, fork) => {
                        f(::std::format_args!(
                            ::std::concat!("{:>ws$}", $ne, "{:", $ew, ">shift$}"),
                            "",
                            "",
                            ws = ws,
                            shift = ($gutter_width + 1) * shift + $gutter_width
                        ))?;
                        for _ in 0..fork {
                            f(::std::format_args!(::std::concat!($sew, "{:", $ew, ">w$}"), "", w = $gutter_width))?;
                        }
                        f(::std::format_args!($sw))
                    }

                    $crate::writer::Branch::ForkLeft(fork) => {
                        f(::std::format_args!(
                            ::std::concat!("{:>ws$}", $se),
                            "",
                            ws = ws
                        ))?;
                        for _ in 0..fork {
                            f(::std::format_args!(
                                ::std::concat!("{:", $ew, ">gutter$}", $sew),
                                "",
                                gutter = $gutter_width,
                            ))?;
                        }
                        f(::std::format_args!(
                            ::std::concat!("{:", $ew, ">gutter$}", $nsw),
                            "",
                            gutter = $gutter_width,
                        ))
                    }
                    $crate::writer::Branch::ForkRight(fork) => {
                        f(::std::format_args!(
                            ::std::concat!("{:>ws$}", $nse),
                            "",
                            ws = ws
                        ))?;
                        for _ in 0..fork {
                            f(::std::format_args!(
                                ::std::concat!("{:", $ew, ">gutter$}", $sew),
                                "",
                                gutter = $gutter_width,
                            ))?;
                        }
                        f(::std::format_args!(
                            ::std::concat!("{:", $ew, ">gutter$}", $sw),
                            "",
                            gutter = $gutter_width,
                        ))
                    }
                    $crate::writer::Branch::ForkMiddle(l, r) => {
                        f(::std::format_args!(::std::concat!("{:ws$}", $se), "", ws = ws))?;
                        for _ in 0..l {
                            f(::std::format_args!(
                                ::std::concat!("{:", $ew, ">gutter$}", $sew),
                                "",
                                gutter = $gutter_width,
                            ))?;
                        }
                        f(::std::format_args!(
                                ::std::concat!("{:", $ew, ">gutterl$}", $nsew, "{:", $ew, ">gutterr$}"),
                                "",
                                "",
                                gutterl = $gutter_width,
                                gutterr = $gutter_width,
                        ))?;
                        for _ in 0..r {
                            f(::std::format_args!(
                                ::std::concat!($sew, "{:", $ew, ">gutter$}"),
                                "",
                                gutter = $gutter_width,
                            ))?;
                        }
                        f(::std::format_args!($sw))
                    }
                    $crate::writer::Branch::MergeLeft => f(::std::format_args!(::std::concat!("{:>ws$}", $ew), "", ws = ws)),
                    $crate::writer::Branch::MergeCenter => f(::std::format_args!(::std::concat!("{:>ws$}", $new), "", ws = ws)),
                    $crate::writer::Branch::MergeRight => f(::std::format_args!(::std::concat!("{:>ws$}", $ne), "", ws = ws)),
                }
            }

            fn write_traverse<F>(mut f: F, count: usize) -> ::std::io::Result<()>
            where
                F: for<'a> FnMut(::std::fmt::Arguments<'a>) -> ::std::io::Result<()>,
            {
                f(::std::format_args!(::std::concat!("{:", $ew, ">tr$}"), "", tr = $gutter_width * (count + 1) + count))
            }
        }
    };
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $new:literal, $nsew:literal$(,)?],
            gutter_width: $gutter_width:expr$(,)?
        }
    ) => {
        $crate::writer::branch_writer! {
            $(#[$outer])*
            $vis struct $name {
                charset: [$ns, $ew, $sw, $se, $nw, $ne, $nsw, $nse, $sew, $new, $nsew],
                gutter_width: $gutter_width,
                inverted: false,
            }
        }
    };
    // FIXME: remove this some time later
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal$(,)?],
            gutter_width: $gutter_width:expr,
            inverted: $inverted:expr$(,)?
        }
    ) => {
        compile_error!("This macro has been changed to require an extra '┴' character in the second last position, before '┼'.");
    };
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            charset: [$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $new:literal, $nsew:literal$(,)?],
            gutter_width: $gutter_width:expr$(,)?
        }
    ) => {
        compile_error!("This macro has been changed to require an extra '┴' character in the second last position, before '┼'.");
    };
}

pub use branch_writer;
