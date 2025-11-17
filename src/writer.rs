use std::{fmt, io};

use crate::config::Config;

pub struct DiagramWriter<W> {
    /// Writer configuration.
    pub config: Config,
    writer: W,
    line_width: usize,
}

impl<W: io::Write> DiagramWriter<W> {
    /// Create a new [`DiagramWriter`] wrapping a writer, using the default configuration.
    pub fn with_default_config(writer: W) -> Self {
        Self {
            config: Config::new(),
            writer,
            line_width: 0,
        }
    }

    /// Returns the number of characters written since the last line break.
    pub fn line_char_count(&self) -> usize {
        self.line_width
    }

    /// Write a [`Branch`].
    #[inline]
    pub fn write_branch(&mut self, b: Branch) -> io::Result<()> {
        let f = &mut self.writer;
        let width = match b {
            Branch::Continue => {
                write!(f, "│")?;
                1
            }
            Branch::Blank(spaces) => {
                write!(f, "{:>branch$}", "", branch = spaces)?;
                spaces
            }
            Branch::ShiftForkLeft(shift, branch) => {
                write!(
                    f,
                    "╭{:┬>branch$}{:─>shift$}╯",
                    "",
                    "",
                    branch = branch,
                    shift = shift
                )?;
                2 + shift + branch
            }
            Branch::ShiftForkRight(shift, branch) => {
                write!(
                    f,
                    "╰{:─>shift$}{:┬>branch$}╮",
                    "",
                    "",
                    branch = branch,
                    shift = shift
                )?;
                2 + shift + branch
            }
            Branch::ForkLeft(branch) => {
                write!(f, "╭{:┬>branch$}┤", "", branch = branch)?;
                2 + branch
            }
            Branch::ForkRight(branch) => {
                write!(f, "├{:┬>branch$}╮", "", branch = branch)?;
                2 + branch
            }
            Branch::ForkMiddle(left, right) => {
                write!(
                    f,
                    "╭{:┬>left$}┼{:┬>right$}╮",
                    "",
                    "",
                    left = left,
                    right = right
                )?;
                3 + left + right
            }
        };
        self.line_width += width;
        Ok(())
    }

    /// Write a vertex marker.
    #[inline]
    pub fn write_vertex_marker(&mut self, marker: char) -> io::Result<()> {
        self.line_width += 1;
        write!(&mut self.writer, "{marker}")
    }

    /// Write a newline.
    #[inline]
    pub fn write_newline(&mut self) -> io::Result<()> {
        self.line_width = 0;
        writeln!(&mut self.writer)
    }

    /// Write a single line of annotation, followed by a newline.
    ///
    /// The caller must guarantee the provided line does not contain any newlines.
    #[inline]
    pub fn write_annotation_line(
        &mut self,
        line: impl fmt::Display,
        bound: usize,
    ) -> io::Result<()> {
        writeln!(
            &mut self.writer,
            "{:>align$}{:>padding$}{line}",
            "",
            "",
            align = bound.saturating_sub(self.line_width),
            padding = self.config.annotation_margin_left
        )?;
        self.line_width = 0;
        Ok(())
    }
}

/// The components from which a branch diagram is created.
#[derive(Debug, Clone, Copy)]
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

impl Branch {
    /// Shorthand for moving `n` columns to the left.
    ///
    /// If `n = 0`, this is a [`Branch::Continue`], and otherwise a
    /// [`Branch::ShiftForkLeft`] with arguments `n` and `0`.
    pub const fn shift_left(n: usize) -> Self {
        match n.checked_sub(1) {
            None => Branch::Continue,
            Some(shift) => Branch::ShiftForkLeft(shift, 0),
        }
    }

    /// Shorthand for moving `n` columns to the right.
    ///
    /// If `n = 0`, this is a [`Branch::Continue`], and otherwise a
    /// [`Branch::ShiftForkRight`] with arguments `n` and `0`.
    pub const fn shift_right(n: usize) -> Self {
        match n.checked_sub(1) {
            None => Branch::Continue,
            Some(shift) => Branch::ShiftForkRight(shift, 0),
        }
    }

    /// Shorthand for forking with `l` columns to the left and `r` columns to the right.
    ///
    /// - If `l = r = 0`, this is a [`Branch::Continue`].
    /// - If `l > 0` and `r = 0` this is a [`Branch::ForkLeft`].
    /// - If `l = 0` and `r > 0` this is a [`Branch::ForkRight`].
    /// - If `l > 0` and `r > 0` this is a [`Branch::ForkMiddle`].
    pub const fn fork(l: usize, r: usize) -> Self {
        match (l.checked_sub(1), r.checked_sub(1)) {
            (None, None) => Branch::Continue,
            (Some(extra), None) => Branch::ForkLeft(extra),
            (None, Some(extra)) => Branch::ForkRight(extra),
            (Some(extra_l), Some(extra_r)) => Branch::ForkMiddle(extra_l, extra_r),
        }
    }
}
