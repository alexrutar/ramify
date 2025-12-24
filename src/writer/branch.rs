/// A single component used when drawing a part of the diagram outside a merge.
///
/// See the documentation for [`DiagramWrite`](crate::writer::DiagramWrite) for more detail on when
/// this struct is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    /// A vertex marker character.
    Marker(char),
    /// A `│` continuation.
    Continue,
    /// A `╭┬─╯` left shift.
    ///
    /// The first field is the number of horizontal spacers and the second field
    /// is the number of new branches.
    ShiftForkLeft(usize, usize),
    /// A `╰─┬╮` right shift and fork.
    ///
    /// The first field is the number of `─` horizontal spacers and the second field
    /// is the number of `┬` new branches.
    ShiftForkRight(usize, usize),
    /// A `╭┬┤` left fork.
    ///
    /// The field is the number of `┬` forks.
    ForkLeft(usize),
    /// A `├┬╮` right fork.
    ///
    /// The field is the number of `┬` forks.
    ForkRight(usize),
    /// A `╭┬┼┬╮` middle fork.
    ///
    /// The first field is the number of `┬` forks on the left, and the second is the
    /// number of `┬` forks on the right.
    ForkMiddle(usize, usize),
    /// A `├` merge starter.
    MergeStart,
    /// A `╭┬─┴` left shift fork merge starter.
    ///
    /// The first field is the number of `─` horizontal spacers and the second field is the number
    /// of `┬` forks.
    ShiftForkLeftMergeStart(usize, usize),
    /// A `╰─┬┬` right shift fork merge starter.
    ///
    /// The first field is the number of `─` horizontal spacers and the second field is the number
    /// of `┬` forks.
    ShiftForkRightMergeStart(usize, usize),
}

/// A single component used when drawing a part of the diagram within a merge.
///
/// See the documentation for [`DiagramWrite`](crate::writer::DiagramWrite) for more detail on when
/// this struct is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeBranch {
    /// A `┴` central merge joiner.
    Join,
    /// A `│` or `─` crossing.
    ///
    /// This is a horizontal line crossing a vertical line, used when a merge line must pass over
    /// (or under) another line.
    Cross,
    /// A `╯` merge end.
    End,
}
