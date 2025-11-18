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

// TODO: Make this line-buffered?

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

    fn branch_width(&self, b: &Branch) -> usize {
        match b {
            Branch::Continue => 1,
            Branch::Blank(spaces) => *spaces,
            Branch::ShiftLeft(shift) | Branch::ShiftRight(shift) => 2 + shift,
            Branch::ForkDoubleLeft | Branch::ForkDoubleRight => 2,
            Branch::ForkDoubleShiftLeft(shift) | Branch::ForkDoubleShiftRight(shift) => 3 + shift,
            Branch::ForkTripleShiftLeft(shift) | Branch::ForkTripleShiftRight(shift) => 4 + shift,
            Branch::ForkTripleLeft | Branch::ForkTripleMiddle | Branch::ForkTripleRight => 3,
        }
    }

    fn write_branch_smooth(&mut self, b: Branch) -> io::Result<()> {
        self.line_width += self.branch_width(&b);
        let f = &mut self.writer;
        match b {
            Branch::Blank(spaces) => write!(f, "{:>branch$}", "", branch = spaces),

            Branch::ShiftLeft(shift) => {
                write!(f, "╭{:─>shift$}╯", "", shift = shift)
            }
            Branch::Continue => write!(f, "│"),
            Branch::ShiftRight(shift) => {
                write!(f, "╰{:─>shift$}╮", "", shift = shift)
            }

            Branch::ForkDoubleShiftLeft(shift) => {
                write!(f, "╭┬{:─>shift$}╯", "", shift = shift)
            }
            Branch::ForkDoubleLeft => {
                write!(f, "╭┤")
            }
            Branch::ForkDoubleRight => {
                write!(f, "├╮")
            }
            Branch::ForkDoubleShiftRight(shift) => {
                write!(f, "╰{:─>shift$}┬╮", "", shift = shift)
            }

            Branch::ForkTripleShiftLeft(shift) => {
                write!(f, "╭┬┬{:─>shift$}╯", "", shift = shift)
            }
            Branch::ForkTripleLeft => {
                write!(f, "╭┬┤")
            }
            Branch::ForkTripleMiddle => {
                write!(f, "╭┼╮")
            }
            Branch::ForkTripleRight => {
                write!(f, "├┬╮")
            }
            Branch::ForkTripleShiftRight(shift) => {
                write!(f, "╰{:─>shift$}┬┬╮", "", shift = shift)
            }
        }
    }

    fn write_branch_sharp(&mut self, b: Branch) -> io::Result<()> {
        self.line_width += self.branch_width(&b);
        let f = &mut self.writer;
        match b {
            Branch::Continue => write!(f, "│"),
            Branch::Blank(spaces) => write!(f, "{:>branch$}", "", branch = spaces),
            _ => todo!(),
        }
    }

    /// Write a [`Branch`].
    #[inline]
    pub(crate) fn write_branch(&mut self, b: Branch) -> io::Result<()> {
        if self.config.compatible_box_chars {
            self.write_branch_sharp(b)
        } else {
            self.write_branch_smooth(b)
        }
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
#[allow(unused)]
pub enum Branch {
    /// A ` ` blank.
    ///
    /// The field is the number of blank spaces.
    Blank(usize),
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
