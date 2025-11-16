pub struct Config {
    /// The margin between each annotation.
    pub annotation_margin_below: usize,
    /// The margin between the annotation and the branch diagram.
    pub annotation_margin_left: usize,
    /// The character set for the edges in the branch diagram.
    pub charset: Charset,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            annotation_margin_below: 0,
            annotation_margin_left: 1,
            charset: Default::default(),
        }
    }
}

/// A set of characters used to make line drawings.
///
/// Two charsets are provided: a [`smooth_corners`](Self::smooth_corners) and a
/// [`sharp_corners`](Self::sharp_corners) charset. The [box
/// drawing](https://en.wikipedia.org/wiki/Box_Drawing) Unicode block can be used to build
/// different character sets.
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
}

impl Default for Charset {
    fn default() -> Self {
        Self::smooth_corners()
    }
}

impl Charset {
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
