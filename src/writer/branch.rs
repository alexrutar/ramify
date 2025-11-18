/// The components used to draw a branch diagram is created.
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Branch {
    /// A `╭╯` left shift.
    ///
    /// The field is the number of extra horizontal spacers.
    ShiftLeft(usize),
    /// A `│` continuation.
    Continue,
    /// A `╰╮` left shift.
    ///
    /// The field is the number of extra horizontal spacers.
    ShiftRight(usize),
    /// A `╭┬╯` left shift and double fork.
    ///
    /// The field is the number of extra horizontal spacers.
    ForkDoubleShiftLeft(usize),
    /// A `╭┤` left double fork.
    ForkDoubleLeft,
    /// A `├╮` left double fork.
    ///
    /// The field is the number of extra horizontal spacers.
    ForkDoubleRight,
    /// A `╰┬╮` right shift and double fork
    ///
    /// The field is the number of extra horizontal spacers.
    ForkDoubleShiftRight(usize),
    /// A `╭┬┬╯` left shift and triple fork.
    ///
    /// The field is the number of extra horizontal spacers.
    ForkTripleShiftLeft(usize),
    /// A `╭┬┤` left triple fork.
    ForkTripleLeft,
    /// A `╭┼╮` middle triple fork.
    ForkTripleMiddle,
    /// A `├┬╮` right triple fork.
    ForkTripleRight,
    /// A `╰┬┬╮` right shift and triple fork.
    ///
    /// The field is the number of extra horizontal spacers.
    ForkTripleShiftRight(usize),
}

macro_rules! branch_writer_impl {
    (
        $(#[$outer:meta])*
        pub struct $name:ident;

        chars => {$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal},
        align => {$wide:expr, $mul:expr, $pos:expr},
        shift => $shift:expr
    ) => {
        $(#[$outer])*
        pub struct $name;

        impl crate::writer::BranchWrite for $name {
            const WIDE: bool = $wide;

            fn write_branch<F>(f: F, ws: usize, b: Branch) -> io::Result<()>
            where
                F: for<'a> FnOnce(fmt::Arguments<'a>) -> io::Result<()>,
            {
                let args = match b {
                    Branch::ShiftLeft(shift) => {
                        format_args!(
                            concat!("{:>ws$}", $se, "{:", $shift, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                    Branch::Continue => format_args!(concat!("{:>ws$}", $ns), "", ws = ws),
                    Branch::ShiftRight(shift) => {
                        format_args!(
                            concat!("{:>ws$}", $ne, "{:", $shift, ">shift$}", $sw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }

                    Branch::ForkDoubleShiftLeft(shift) => {
                        format_args!(
                            concat!("{:>ws$}", $se, $ew, $sew, "{:", $shift, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                    Branch::ForkDoubleLeft => {
                        format_args!(concat!("{:>ws$}", $se, $ew, $nsw), "", ws = ws)
                    }
                    Branch::ForkDoubleRight => {
                        format_args!(concat!("{:>ws$}", $nse, $ew, $sw), "", ws = ws)
                    }
                    Branch::ForkDoubleShiftRight(shift) => {
                        format_args!(
                            concat!("{:>ws$}", $nw, "{:", $shift, ">shift$}", $sew, $ew, $sw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }

                    Branch::ForkTripleShiftLeft(shift) => {
                        format_args!(
                            concat!("{:>ws$}", $se, $ew, $sew, $ew, $sew, "{:", $shift, ">shift$}", $nw),
                            "",
                            "",
                            ws = ws,
                            shift = $mul * shift + $pos
                        )
                    }
                    Branch::ForkTripleLeft => {
                        format_args!(concat!("{:>ws$}", $se, $ew, $sew, $ew, $nsw), "", ws = ws)
                    }
                    Branch::ForkTripleMiddle => {
                        format_args!(concat!("{:>ws$}", $se, $ew, $nsew, $ew, $sw), "", ws = ws)
                    }
                    Branch::ForkTripleRight => {
                        format_args!(concat!("{:>ws$}", $nse, $ew, $sew, $ew, $sw), "", ws = ws)
                    }
                    Branch::ForkTripleShiftRight(shift) => {
                        format_args!(
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

/// TODO
#[macro_export]
macro_rules! branch_writer {
    (
        $(#[$outer:meta])*
        pub struct $name:ident;

        charset => {$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal},
        wide => false
    ) => {
        branch_writer_impl!(
            $(#[$outer])*
            pub struct $name;

            chars => {$ns, "", $sw, $se, $nw, $ne, $nsw, $nse, $sew, $nsew},
            align => {false, 1, 0},
            shift => $ew
        );
    };
    (
        $(#[$outer:meta])*
        pub struct $name:ident;

        charset => {$ns:literal, $ew:literal, $sw:literal, $se:literal, $nw:literal, $ne:literal, $nsw:literal, $nse:literal, $sew:literal, $nsew:literal},
        wide => true
    ) => {
        branch_writer_impl!(
            $(#[$outer])*
            pub struct $name;

            chars => {$ns, $ew, $sw, $se, $nw, $ne, $nsw, $nse, $sew, $nsew},
            align => {true, 2, 1},
            shift => $ew
        );
    };
}

pub use branch_writer;
pub(crate) use branch_writer_impl;
