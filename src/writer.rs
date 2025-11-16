use std::{fmt, io};

use crate::config::Config;

pub struct DiagramWriter<W> {
    pub config: Config,
    writer: W,
}

impl<W: io::Write> DiagramWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            config: Config::default(),
            writer,
        }
    }

    #[inline]
    pub fn branch(&mut self, b: Branch) -> io::Result<()> {
        write!(&mut self.writer, "{b}")
    }

    #[inline]
    pub fn mark(&mut self, marker: char) -> io::Result<()> {
        write!(&mut self.writer, "{marker}")
    }

    #[inline]
    pub fn newline(&mut self) -> io::Result<()> {
        writeln!(&mut self.writer, "")
    }

    #[inline]
    pub fn annotation_line(&mut self, line: impl fmt::Display) -> io::Result<()> {
        writeln!(
            &mut self.writer,
            "{:>padding$}{line}",
            "",
            padding = self.config.annotation_margin_left
        )
    }
}

pub enum Branch {
    /// A `│` continuation.
    Continue,

    /// A ` ` blank.
    ///
    /// - The field is the number of blank spaces.
    Blank(usize),

    /// A `╭┤` fork.
    ///
    /// - The field is the number of extra `┬` forks.
    ///
    /// For example, `ForkLeft(2)` is `╭┬┬┤`.
    ForkLeft(usize),

    /// A `├╮` fork.
    ///
    /// - The field is the number of extra `┬` forks.
    ///
    /// For example, `ForkRight(1)` is `├┬╮`.
    ForkRight(usize),

    /// A `╭┼╮` fork.
    ///
    /// - The first field is the number of extra `┬` forks on the left.
    /// - The second field is the number of extra `┬` forks on the right.
    ///
    /// For example, `ForkMiddle(1, 2)` is `╭┬┼┬┬╮`.
    ForkMiddle(usize, usize),

    /// A `╭╯` combined shift and fork.
    ///
    /// - The first field is the number of horizontal `─` spacers.
    /// - The second field is the number of `┬` forks.
    ///
    /// For example, `ShiftForkLeft(2, 1)` is `╭┬──╯`.
    ShiftForkLeft(usize, usize),

    /// A `╰╮` combined shift and fork.
    ///
    /// - The first field is the number of horizontal `─` spacers.
    /// - The second field is the number of `┬` forks.
    ///
    /// For example, `ShiftForkRight(1, 2)` is `╰─┬┬╮`.
    ShiftForkRight(usize, usize),
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Branch::Continue => {
                write!(f, "│")
            }
            Branch::Blank(spaces) => {
                write!(f, "{:>branch$}", "", branch = spaces)
            }
            Branch::ShiftForkLeft(shift, branch) => {
                write!(
                    f,
                    "╭{:┬>branch$}{:─>shift$}╯",
                    "",
                    "",
                    branch = branch,
                    shift = shift
                )
            }
            Branch::ShiftForkRight(shift, branch) => {
                write!(
                    f,
                    "╰{:─>shift$}{:┬>branch$}╮",
                    "",
                    "",
                    branch = branch,
                    shift = shift
                )
            }
            Branch::ForkLeft(branch) => {
                write!(f, "╭{:┬>branch$}┤", "", branch = branch)
            }
            Branch::ForkRight(branch) => {
                write!(f, "├{:┬>branch$}╮", "", branch = branch)
            }
            Branch::ForkMiddle(left, right) => {
                write!(
                    f,
                    "╭{:┬>left$}┼{:┬>right$}╮",
                    "",
                    "",
                    left = left,
                    right = right
                )
            }
        }
    }
}

impl Branch {
    pub const fn shift_left(n: usize) -> Self {
        match n.checked_sub(1) {
            None => Branch::Continue,
            Some(shift) => Branch::ShiftForkLeft(shift, 0),
        }
    }

    pub const fn shift_right(n: usize) -> Self {
        match n.checked_sub(1) {
            None => Branch::Continue,
            Some(shift) => Branch::ShiftForkRight(shift, 0),
        }
    }

    pub const fn fork(l: usize, r: usize) -> Self {
        match (l.checked_sub(1), r.checked_sub(1)) {
            (None, None) => Branch::Continue,
            (Some(extra), None) => Branch::ForkLeft(extra),
            (None, Some(extra)) => Branch::ForkRight(extra),
            (Some(extra_l), Some(extra_r)) => Branch::ForkMiddle(extra_l, extra_r),
        }
    }
}
