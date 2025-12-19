//! # Layout configuration

use crate::Generator;

/// Layout configuration used by a [`Generator`](crate::Generator) to control the branch diagram
/// generation algorithm.
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
    pub const fn minimize_width(mut self, minimize: bool) -> Self {
        self.minimize_width = minimize;
        self
    }

    /// Write the vertex on the last row of the annotation instead of the first.
    ///
    /// This is mostly useful for writing the tree with the root at the bottom.
    ///
    /// This will result in slightly taller branch diagrams. Even if you are writing in
    /// inverted mode, if your annotations occupy at most one line, it can be useful
    /// to keep this as `false` anyway.
    ///
    /// The default value is `false`.
    pub const fn annotation_before_vertex(mut self, before: bool) -> Self {
        self.annotation_before_vertex = before;
        self
    }

    /// Write the annotation lines in reverse order.
    ///
    /// This is mostly useful for writing the tree with the root at the bottom.
    ///
    /// The default value is `false`.
    pub const fn reverse_annotation_lines(mut self, rev: bool) -> Self {
        self.reverse_annotation_lines = rev;
        self
    }

    /// Set standard defaults for inverted drawing.
    ///
    /// The default value is `false`.
    pub const fn inverted_annotations(self, invert: bool) -> Self {
        self.annotation_before_vertex(invert)
            .reverse_annotation_lines(invert)
    }
}
