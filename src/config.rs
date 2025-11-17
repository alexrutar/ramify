pub struct Config {
    /// The margin between each annotation.
    pub annotation_margin_below: usize,
    /// The margin between the annotation and the branch diagram.
    pub annotation_margin_left: usize,
    /// The character set for the edges in the branch diagram.
    pub charset: Charset,
}

impl Config {
    /// Initialize configuration using default values.
    ///
    /// This is the same as the [`Default`] implementation, but as a `const fn`.
    pub const fn new() -> Self {
        Self {
            annotation_margin_below: 0,
            annotation_margin_left: 1,
            charset: Charset::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of characters used to make line drawings.
///
/// Two charsets are provided: a [`smooth_corners`](Self::smooth_corners) and a
/// [`sharp_corners`](Self::sharp_corners) charset. The [box
/// drawing](https://en.wikipedia.org/wiki/Box_Drawing) Unicode block can be used to build
/// different character sets.
///
/// Most characters are self-explanatory, but there are two notes.
///
/// 1. The [`internal_whitespace`](Self::internal_whitespace) character is used to write whitespace
///    which is considered to be part of the branch diagram. For example, it is written when
///    a row is skipped because it will be occupied by a future row, to prevent jitter.
///    Occasionally, this character is written at the end of a row. For example, if a 3-way fork
///    cannot be fully written, it will still reserve space for itself by writing a whitespace
///    character. If the fork cannot be fully written because of row width limits, it will still
///    partially write whitespace characters.
///
/// 2. The [`up_and_horizontal`](Self::up_and_horizontal) character is never used in the standard
///    top-down printing. However, it is used for vertical reflections, which are necessary to
///    print trees "upside down".
pub struct Charset {
    /// The '│' character.
    pub vertical: char,
    /// The '┼' character.
    pub vertical_and_horizontal: char,
    /// The '┤' character.
    pub vertical_and_left: char,
    /// The '├' character.
    pub vertical_and_right: char,
    /// The '╮' character.
    pub down_and_left: char,
    /// The '╭' character.
    pub down_and_right: char,
    /// The '┬' character.
    pub down_and_horizontal: char,
    /// The '╯' character.
    pub up_and_left: char,
    /// The '╰' character.
    pub up_and_right: char,
    /// The '┴' character.
    pub up_and_horizontal: char,
    /// The '─' character.
    pub horizontal: char,
    /// The ` ` character.
    pub internal_whitespace: char,
}

impl Default for Charset {
    fn default() -> Self {
        Self::new()
    }
}

impl Charset {
    const fn new() -> Self {
        Self::smooth_corners()
    }

    /// The default charset, which has smooth corners.
    /// ```txt
    /// ╯ ┴ ╰ ─
    /// ┤ ┼ ├ │
    /// ╮ ┬ ╭
    /// ```
    pub const fn smooth_corners() -> Self {
        Self {
            vertical_and_right: '├',
            vertical_and_left: '┤',
            vertical_and_horizontal: '┼',
            down_and_horizontal: '┬',
            up_and_horizontal: '┴',
            down_and_right: '╭',
            down_and_left: '╮',
            up_and_left: '╯',
            up_and_right: '╰',
            horizontal: '─',
            vertical: '│',
            internal_whitespace: ' ',
        }
    }

    /// A charset with sharp corners.
    /// ```txt
    /// ┘ ┴ └ ─
    /// ┤ ┼ ├ │
    /// ┐ ┬ ┌
    /// ```
    pub const fn sharp_corners() -> Self {
        Self {
            down_and_left: '┐',
            down_and_right: '┌',
            up_and_left: '┘',
            up_and_right: '└',
            ..Self::smooth_corners()
        }
    }
}
