//! Diagram writer styles

use std::{fmt, io};

use super::{FmtWriter, IOWriter};

/// Configuration for the appearance of the branch diagram.
///
/// This struct is used by [`IOWriter`]s and [`FmtWriter`]s.
/// For more fine-grained configuration than is available here, one should manually
/// implement [`DiagramWrite`](super::DiagramWrite).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    /// A set of characters to use in the diagram.
    pub(crate) charset: Charset,
    /// The margin at the beginning of each annotation.
    pub(crate) annotation_margin: usize,
    /// The minimum left alignment of each annotation line.
    pub(crate) annotation_justification: usize,
    /// The gap between vertices.
    pub(crate) gutter_width: usize,
    /// Put merge lines on top.
    pub(crate) merge_over: bool,
}

impl Style {
    /// Initialize a new style from the provided character set.
    #[inline]
    pub const fn new(charset: Charset) -> Self {
        Self {
            charset,
            annotation_margin: 1,
            annotation_justification: 0,
            gutter_width: 0,
            merge_over: false,
        }
    }

    /// Get an [`IOWriter`] using this style.
    #[inline]
    pub const fn io_writer<W: io::Write>(self, writer: W) -> IOWriter<W> {
        IOWriter::new(self, writer)
    }

    /// Get a [`FmtWriter`] using this style.
    #[inline]
    pub const fn fmt_writer<W: fmt::Write>(self, writer: W) -> FmtWriter<W> {
        FmtWriter::new(self, writer)
    }

    /// Initialize with the [rounded corners](Charset::rounded_corners) character set.
    ///
    /// The charset looks as follows.
    /// ```txt
    /// 0    
    /// ├┬╮  
    /// │1├╮
    /// ││2│
    /// │3│├╮
    /// │╭╯││
    /// ││╭┤│
    /// │││4│
    /// ││5╭╯
    /// │6╭╯
    /// 7╭╯  
    ///  8   
    /// ```
    #[inline]
    pub const fn rounded_corners() -> Self {
        Self::new(Charset::rounded_corners())
    }

    /// Initialize with the [sharp corners](Charset::sharp_corners) character set.
    ///
    /// The charset looks as follows.
    /// ```txt
    /// 0
    /// ├┬┐
    /// │1├┐
    /// ││2│
    /// │3│├┐
    /// │┌┘││
    /// ││┌┤│
    /// │││4│
    /// ││5┌┘
    /// │6┌┘
    /// 7┌┘
    ///  8
    /// ```
    #[inline]
    pub const fn sharp_corners() -> Self {
        Self::new(Charset::sharp_corners())
    }

    /// Initialize with the [doubled lines](Charset::doubled_lines) character set.
    ///
    /// The charset looks as follows.
    /// ```txt
    /// 0
    /// ╠╦╗
    /// ║1╠╗
    /// ║║2║
    /// ║3║╠╗
    /// ║╔╝║║
    /// ║║╔╣║
    /// ║║║4║
    /// ║║5╔╝
    /// ║6╔╝
    /// 7╔╝
    ///  8
    /// ```
    #[inline]
    pub const fn doubled_lines() -> Self {
        Self::new(Charset::doubled_lines())
    }

    /// Reset the character set used in the style.
    #[inline]
    pub const fn charset(mut self, charset: Charset) -> Self {
        self.charset = charset;
        self
    }

    /// Invert the internal character set.
    ///
    /// This inverts vertically by swapping characters like `╯` and `╮`. Calling this method twice
    /// will return the original character set. This makes the lines connect correctly when writing
    /// the diagram lines in reverse order.
    ///
    /// Also see [`Config::inverted_annotations`](crate::Config::inverted_annotations) to modify the layout algorithm to
    /// draw annotations correctly if you are also rendering annotations.
    ///
    /// ## Example
    /// Inverting has the following effect (the third column has reversed lines).
    /// ```txt
    /// 0         0          8
    /// ├┬╮       ├┴╯       7╰╮
    /// │1├╮      │1├╯      │6╰╮
    /// ││2│      ││2│      ││5╰╮
    /// │3│├╮     │3│├╯     │││4│
    /// │╭╯││  →  │╰╮││  ↺  ││╰┤│
    /// ││╭┤│     ││╰┤│     │╰╮││
    /// │││4│  →  │││4│  ↺  │3│├╯
    /// ││5╭╯     ││5╰╮     ││2│
    /// │6╭╯      │6╰╮      │1├╯
    /// 7╭╯       7╰╮       ├┴╯
    ///  8         8        0
    /// ```
    #[inline]
    pub const fn invert(mut self) -> Self {
        self.charset = self.charset.invert();
        self
    }

    /// The margin at the beginning of each annotation.
    ///
    /// This is the number of characters written at the beginning of the annotation line to create
    /// a gap between the annotation and the branch diagram lines.
    ///
    /// The default value is `1`.
    ///
    /// ## Example
    /// Here is an example going from annotation margin `0` to `2`.
    /// ```txt
    /// 0             0
    /// ├┬╮           ├┬╮
    /// │1├╮L0    →   │1├╮  L0
    /// ││││L1        ││││  L1
    /// ││2│          ││2│
    /// │3│├╮L0   →   │3│├╮  L0
    /// │╭╯││         │╭╯││
    /// ││╭┤│         ││╭┤│
    /// │││4│L0   →   │││4│  L0
    /// │││╭╯L1       │││╭╯  L1
    /// ││││ L2       ││││   L2
    /// ││5│          ││5│
    /// │6╭╯          │6╭╯
    /// 7╭╯           7╭╯
    ///  8             8
    /// ```
    #[inline]
    pub const fn annotation_margin(mut self, annotation_margin: usize) -> Self {
        self.annotation_margin = annotation_margin;
        self
    }

    /// The minimum left justification of the annotations.
    ///
    /// Annotation lines will never begin earlier than this value, but may begin later than this
    /// value if the row width plus annotation margin exceeds the justification. Note that the
    /// justficiation is the number of characters, rather than columns, and therefore may need to be
    /// adjusted if the gutter width is not zero.
    ///
    /// The default value is `0`.
    ///
    /// ## Example
    /// Here is an example going from justification 0 to 5, with an annotation margin of 1.
    /// Observe that:
    ///
    /// - `L0` is not shifted, since the justification refers to the start of the line after the
    ///   margin
    /// - `L1` still starts 1 character to the right of the justification since the diagram is too wide.
    /// - `L2` is shifted right by two characters, so that it begins 5 characters from the
    ///   boundary.
    /// ```txt
    /// 0             0
    /// ├┬╮           ├┬╮
    /// │1├╮ L0   →   │1├╮ L0
    /// ││2│          ││2│
    /// │3│├╮         │3│├╮
    /// │╭╯││         │╭╯││
    /// ││╭┤│         ││╭┤│
    /// │││4│ L1  →   │││4│ L1
    /// ││5╭╯         ││5╭╯
    /// │6╭╯          │6╭╯
    /// 7╭╯           7╭╯
    ///  8 L2     →    8   L2
    ///               ---->
    /// ```
    #[inline]
    pub const fn annotation_justification(mut self, annotation_justification: usize) -> Self {
        self.annotation_justification = annotation_justification;
        self
    }

    /// The gap between columns in the branch diagram.
    ///
    /// The default value is `0`.
    ///
    /// ## Example
    /// Setting the gutter width to `1` has the following effect.
    /// ```txt
    /// 0         0     
    /// ├┬╮       ├─┬─╮  
    /// │1├╮      │ 1 ├─╮
    /// ││2│      │ │ 2 │
    /// │3│├╮     │ 3 │ ├─╮
    /// │╭╯││  →  │ ╭─╯ │ │
    /// ││╭┤│     │ │ ╭─┤ │
    /// │││4│  →  │ │ │ 4 │
    /// ││5╭╯     │ │ 5 ╭─╯
    /// │6╭╯      │ 6 ╭─╯
    /// 7╭╯       7 ╭─╯   
    ///  8          8
    ///            ^ ^ ^ ^ gutters
    /// ```
    #[inline]
    pub const fn gutter_width(mut self, gutter_width: usize) -> Self {
        self.gutter_width = gutter_width;
        self
    }

    /// Whether merge lines should go above diagram lines.
    ///
    /// Setting this value to `true` typically requires a
    /// [gutter width](Self::gutter_width) of at least 1 for diagram clarity.
    ///
    /// The default value is `false`.
    ///
    /// ## Example
    /// With a gutter width of 1, setting this to `true` has the following effect:
    /// ```txt
    /// 0             0
    /// ├─╮           ├─╮
    /// │ 1           │ 1
    /// │ ├─╮         │ ├─╮
    /// │ 2 ╰───╮     │ 2 ╰───╮
    /// │ ├─┬─╮ │     │ ├─┬─╮ │
    /// ├─│─╯ │ │  →  ├───╯ │ │
    /// 3 │ ╭─╯ │     3 │ ╭─╯ │
    /// │ │ 4 ╭─╯     │ │ 4 ╭─╯
    /// │ ╰─╮ │       │ ╰─╮ │
    /// ├─╮ │ │       ├─╮ │ │
    /// ├─│─╯ │    →  ├───╯ │
    /// 5 │ ╭─╯       5 │ ╭─╯
    /// ╭─┴─╯         ╭─┴─╯
    /// 6             6
    /// ```
    #[inline]
    pub const fn merge_over(mut self, merge_over: bool) -> Self {
        self.merge_over = merge_over;
        self
    }
}

/// A set of characters to use in the diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charset {
    /// The ` ` character.
    pub space: char,
    /// The `╮` character.
    pub down_and_left: char,
    /// The `╭` character.
    pub down_and_right: char,
    /// The `╯` character.
    pub up_and_left: char,
    /// The `╰` character.
    pub up_and_right: char,
    /// The `─` character.
    pub horizontal: char,
    /// The `│` character.
    pub vertical: char,
    /// The `┬` character.
    pub down_and_horizontal: char,
    /// The `┴` character.
    pub up_and_horizontal: char,
    /// The `┤` character.╮
    pub vertical_and_left: char,
    /// The `├` character.
    pub vertical_and_right: char,
    /// The `┼` character.
    pub vertical_and_horizontal: char,
}

impl Charset {
    /// Using this character set in a diagram style.
    #[inline]
    pub const fn style(self) -> Style {
        Style::new(self)
    }

    /// The rounded corners character set.
    /// ```txt
    /// ╯ ┴ ╰
    /// ┤ ┼ ├ ─
    /// ╮ ┬ ╭ │
    /// ```
    #[inline]
    pub const fn rounded_corners() -> Self {
        Self {
            space: ' ',
            down_and_left: '╮',
            down_and_right: '╭',
            up_and_left: '╯',
            up_and_right: '╰',
            horizontal: '─',
            vertical: '│',
            down_and_horizontal: '┬',
            up_and_horizontal: '┴',
            vertical_and_left: '┤',
            vertical_and_right: '├',
            vertical_and_horizontal: '┼',
        }
    }

    /// The sharp corners character set.
    /// ```txt
    /// ┘ ┴ └
    /// ┤ ┼ ├ ─
    /// ┐ ┬ ┌ │
    /// ```
    #[inline]
    pub const fn sharp_corners() -> Self {
        Self {
            down_and_left: '┐',
            down_and_right: '┌',
            up_and_left: '┘',
            up_and_right: '└',
            ..Self::rounded_corners()
        }
    }

    /// The doubled lines character set.
    /// ```txt
    /// ╝ ╩ ╚
    /// ╣ ╬ ╠ ═
    /// ╗ ╦ ╔ ║
    /// ```
    #[inline]
    pub const fn doubled_lines() -> Self {
        Self {
            space: ' ',
            down_and_left: '╗',
            down_and_right: '╔',
            up_and_left: '╝',
            up_and_right: '╚',
            horizontal: '═',
            vertical: '║',
            down_and_horizontal: '╦',
            up_and_horizontal: '╩',
            vertical_and_left: '╣',
            vertical_and_right: '╠',
            vertical_and_horizontal: '╬',
        }
    }

    /// Flip the orientation of the charset vertically.
    ///
    /// This swaps the `down_*` characters with the `up_*` characters. This is useful when you
    /// want to render the branch diagram with the root at the bottom since
    /// it will make the branch diagram lines connect correctly when the lines are written in
    /// reverse order.
    ///
    /// Also see [`Config::inverted_annotations`](crate::Config::inverted_annotations) to modify the layout algorithm to
    /// draw annotations correctly.
    #[inline]
    pub const fn invert(self) -> Self {
        Self {
            down_and_left: self.up_and_left,
            down_and_right: self.up_and_right,
            down_and_horizontal: self.up_and_horizontal,
            up_and_left: self.down_and_left,
            up_and_right: self.down_and_right,
            up_and_horizontal: self.down_and_horizontal,
            ..self
        }
    }
}
