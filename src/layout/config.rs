//! # Layout configuration

use crate::Generator;

/// Configuration used by a [`Generator`](crate::Generator) to control the layout
/// algorithm.
///
/// For style configuration with a built-in diagram writer, see the [`Style`](crate::writer::Style) struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub(crate) row_padding: usize,
    pub(crate) minimize_width: bool,
    pub(crate) annotation_before_vertex: bool,
    pub(crate) reverse_annotation_lines: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Initialize using default values.
    ///
    /// This is a const function which is identical to the [`Default`] implementation.
    pub const fn new() -> Self {
        Self {
            row_padding: 0,
            minimize_width: false,
            annotation_before_vertex: false,
            reverse_annotation_lines: false,
        }
    }

    /// Construct a generator using this configuration, starting at the provided root and with the
    /// given ramifier.
    ///
    /// This is a convenience wrapper around [`Generator::with_config`].
    pub fn generator<V, R>(self, root: V, ramifier: R) -> Generator<V, R> {
        Generator::with_config(root, ramifier, self)
    }

    /// The number of extra rows to use as padding between vertices.
    ///
    /// The padding applies after the annotation, or after the vertex if there
    /// is no annotation.
    ///
    /// The default is `0`.
    ///
    /// ## Example
    /// Here is an example going from a row padding of 0 to 1.
    /// ```txt
    ///  0         0
    ///  ├┬╮       ├┬╮
    ///  │1├╮      │1├╮
    ///  ││2│      ││││
    ///  │3│├╮  →  ││2│
    ///  │╭╯││     ││││
    ///  ││╭┤│     │3│├╮
    ///  │││4│     │╭╯││
    ///  ││5╭╯  →  ││╭┤│
    ///  │6╭╯      │││4│
    ///  7╭╯       │││╭╯
    ///   8        ││5│
    ///            ││╭╯
    ///            │6│
    ///            │╭╯
    ///            7│
    ///            ╭╯
    ///            8
    /// ```
    pub const fn row_padding(mut self, pad: usize) -> Self {
        self.row_padding = pad;
        self
    }

    /// Whether to minimize the width.
    ///
    /// If set, the preparation rows between vertex markers will also remove
    /// all internal whitespace. This almost always makes the branch diagram taller.
    ///
    /// The default value is `false`.
    ///
    /// ## Example
    /// Here is an example minimizing the width.
    /// ```txt
    ///  0         0
    ///  ├┬╮       ├┬╮
    ///  │1├╮      │1├╮
    ///  ││2│      ││2│
    ///  │3│├╮  →  │3│├╮
    ///  │╭╯││     │╭╯││
    ///  ││╭┤│     ││╭┤│
    ///  │││4│     │││4│
    ///  ││5╭╯  →  │││╭╯
    ///  │6╭╯      ││5│
    ///  7╭╯       ││╭╯
    ///   8        │6│
    ///            │╭╯
    ///            7│
    ///            ╭╯
    ///            8
    /// ```
    pub const fn minimize_width(mut self, minimize: bool) -> Self {
        self.minimize_width = minimize;
        self
    }

    /// Write the vertex on the last row of the annotation instead of the first.
    ///
    /// This is mostly useful for writing the tree with the root at the bottom.
    ///
    /// This can result in slightly taller branch diagrams. In particular, even if you are writing in
    /// inverted mode, if your annotations occupy at most one line, it can be useful
    /// to keep this as `false` anyway.
    ///
    /// The default value is `false`.
    ///
    /// ## Example
    /// Here is an example writing the annotation before the vertex.
    /// ```txt
    /// 0             0
    /// ├┬╮           ├┬╮
    /// │1├╮ L0   →   ││├╮ L0
    /// ││││ L1       │1││ L1
    /// ││2│          ││2│
    /// │3│├╮ L0  →   │3│├╮ L0
    /// │╭╯││         │╭╯││
    /// ││╭┤│         ││╭┤│
    /// │││4│ L0  →   │││││ L0
    /// │││╭╯ L1      │││││ L1
    /// ││││  L2      │││4│ L2
    /// ││5│          ││5╭╯
    /// │6╭╯          │6╭╯
    /// 7╭╯           7╭╯
    ///  8             8
    /// ```
    pub const fn annotation_before_vertex(mut self, before: bool) -> Self {
        self.annotation_before_vertex = before;
        self
    }

    /// Write the annotation lines in reverse order.
    ///
    /// This is mostly useful for writing the tree with the root at the bottom.
    ///
    /// The default value is `false`.
    ///
    /// ## Example
    /// Here is an example writing the annotation lines in reverse order.
    /// ```txt
    /// 0            0       
    /// ├┬╮          ├┬╮     
    /// │1├╮ L0   →  │1├╮ L1
    /// ││││ L1      ││││ L0
    /// ││2│         ││2│    
    /// │3│├╮ L0  →  │3│├╮ L0
    /// │╭╯││        │╭╯││   
    /// ││╭┤│        ││╭┤│   
    /// │││4│ L0  →  │││4│ L2
    /// │││╭╯ L1     │││╭╯ L1
    /// ││││  L2     ││││  L0
    /// ││5│         ││5│    
    /// │6╭╯         │6╭╯    
    /// 7╭╯          7╭╯     
    ///  8            8      
    /// ```
    pub const fn reverse_annotation_lines(mut self, rev: bool) -> Self {
        self.reverse_annotation_lines = rev;
        self
    }

    /// Set standard defaults for inverted drawing.
    ///
    /// This [writes the annotation before the vertex](Self::annotation_before_vertex) and
    /// [reverses the annotation lines](Self::reverse_annotation_lines). This configuration option
    /// is commonly combined with an [inverted character set](crate::writer::Style::invert) and
    /// rendered with the lines in reverse order.
    ///
    /// Note that setting this value will overwrite the values of the [`annotation_before_vertex`](Self::annotation_before_vertex)
    /// and [`reverse_annotation_lines`](Self::reverse_annotation_lines) settings.
    ///
    /// ## Example
    /// When inverted, the tree is transformed as follows.
    /// The third diagram uses an inverted character set and the lines are displayed last to first.
    /// ```txt
    /// 0             0              8
    /// ├┬╮           ├┬╮           7╰╮
    /// │1├╮ L0   →   ││├╮ L1   ↺   │6╰╮
    /// ││││ L1       │1││ L0       ││5╰╮
    /// ││2│          ││2│          │││4│ L0
    /// │3│├╮ L0      │3│├╮ L0      │││││ L1
    /// │╭╯││     →   │╭╯││     ↺   │││││ L2
    /// ││╭┤│         ││╭┤│         ││╰┤│
    /// │││4│ L0      │││││ L2      │╰╮││
    /// │││╭╯ L1      │││││ L1      │3│├╯ L0
    /// ││││  L2  →   │││4│ L0  ↺   ││2│
    /// ││5│          ││5╭╯         │1││ L0
    /// │6╭╯          │6╭╯          ││├╯ L1
    /// 7╭╯           7╭╯           ├┴╯
    ///  8             8            0
    /// ```
    pub const fn inverted_annotations(self, invert: bool) -> Self {
        self.annotation_before_vertex(invert)
            .reverse_annotation_lines(invert)
    }
}
