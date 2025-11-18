use std::{fmt, io};

/// Configuration passed to a [`DiagramWriter`](crate::writer::DiagramWriter) in order to influence
/// the appearance and layout of the branch diagram and associated annotations.
#[derive(Debug, Clone)]
pub struct Config {
    /// The margin between each annotation. The default is `0`.
    pub margin_below: usize,
    /// The margin between the annotation and the branch diagram. The default is `1`.
    pub margin_left: usize,
    /// Use box drawing characters which look a bit worse but have better font support. The default
    /// is `false`.
    pub compatible_box_chars: bool,
}

impl Config {
    /// Initialize configuration using default values.
    ///
    /// This is the same as the [`Default`] implementation.
    pub const fn new() -> Self {
        Self {
            margin_below: 0,
            margin_left: 1,
            compatible_box_chars: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper around an [`io::Write`] implementation which contains configuration relevant for
/// drawing branch diagrams.
///
/// Note that many small calls to `write!` are made during normal running of this program.
/// It is recommended that the output of the internal writer is buffered.
pub struct Writer<W> {
    /// Configuration used when drawing the branch diagram.
    pub config: Config,
    writer: W,
    line_width: usize,
}

impl<W: io::Write> Writer<W> {
    /// Initialize a new diagram writer with the provided configuration and writer.
    pub fn new(config: Config, writer: W) -> Self {
        Self {
            config,
            writer,
            line_width: 0,
        }
    }

    /// Create a new diagram writer wrapping the provided writer, using default configuration.
    pub fn with_default_config(writer: W) -> Self {
        Self {
            config: Config::new(),
            writer,
            line_width: 0,
        }
    }

    /// Returns the number of characters written since the last line break.
    pub(crate) fn line_char_count(&self) -> usize {
        self.line_width
    }

    /// Write a [`Branch`].
    #[inline]
    pub(crate) fn write_branch(&mut self, b: Branch) -> io::Result<()> {
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
                if self.config.compatible_box_chars {
                    write!(
                        f,
                        "┌{:┬>branch$}{:─>shift$}┘",
                        "",
                        "",
                        branch = branch,
                        shift = shift
                    )?;
                } else {
                    write!(
                        f,
                        "╭{:┬>branch$}{:─>shift$}╯",
                        "",
                        "",
                        branch = branch,
                        shift = shift
                    )?;
                }
                2 + shift + branch
            }
            Branch::ShiftForkRight(shift, branch) => {
                if self.config.compatible_box_chars {
                    write!(
                        f,
                        "└{:─>shift$}{:┬>branch$}┐",
                        "",
                        "",
                        branch = branch,
                        shift = shift
                    )?;
                } else {
                    write!(
                        f,
                        "╰{:─>shift$}{:┬>branch$}╮",
                        "",
                        "",
                        branch = branch,
                        shift = shift
                    )?;
                }
                2 + shift + branch
            }
            Branch::ForkLeft(branch) => {
                if self.config.compatible_box_chars {
                    write!(f, "┌{:┬>branch$}┤", "", branch = branch)?;
                } else {
                    write!(f, "╭{:┬>branch$}┤", "", branch = branch)?;
                }
                2 + branch
            }
            Branch::ForkRight(branch) => {
                if self.config.compatible_box_chars {
                    write!(f, "├{:┬>branch$}┐", "", branch = branch)?;
                } else {
                    write!(f, "├{:┬>branch$}╮", "", branch = branch)?;
                }
                2 + branch
            }
            Branch::ForkMiddle(left, right) => {
                if self.config.compatible_box_chars {
                    write!(
                        f,
                        "┌{:┬>left$}┼{:┬>right$}┐",
                        "",
                        "",
                        left = left,
                        right = right
                    )?;
                } else {
                    write!(
                        f,
                        "╭{:┬>left$}┼{:┬>right$}╮",
                        "",
                        "",
                        left = left,
                        right = right
                    )?;
                }
                3 + left + right
            }
        };
        self.line_width += width;
        Ok(())
    }

    /// Write a vertex marker.
    #[inline]
    pub(crate) fn write_vertex_marker(&mut self, marker: char) -> io::Result<()> {
        self.line_width += 1;
        write!(&mut self.writer, "{marker}")
    }

    /// Write a newline.
    #[inline]
    pub(crate) fn write_newline(&mut self) -> io::Result<()> {
        self.line_width = 0;
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
    ) -> io::Result<()> {
        writeln!(
            &mut self.writer,
            "{:>align$}{:>padding$}{line}",
            "",
            "",
            align = bound.saturating_sub(self.line_width),
            padding = self.config.margin_left
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
            None => Self::Continue,
            Some(shift) => Self::ShiftForkLeft(shift, 0),
        }
    }

    /// Shorthand for moving `n` columns to the right.
    ///
    /// If `n = 0`, this is a [`Branch::Continue`], and otherwise a
    /// [`Branch::ShiftForkRight`] with arguments `n` and `0`.
    pub const fn shift_right(n: usize) -> Self {
        match n.checked_sub(1) {
            None => Self::Continue,
            Some(shift) => Self::ShiftForkRight(shift, 0),
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
            (None, None) => Self::Continue,
            (Some(extra), None) => Self::ForkLeft(extra),
            (None, Some(extra)) => Self::ForkRight(extra),
            (Some(extra_l), Some(extra_r)) => Self::ForkMiddle(extra_l, extra_r),
        }
    }
}
