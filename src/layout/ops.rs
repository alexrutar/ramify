#[cfg(test)]
mod tests;

use std::io;

use crate::{
    columns::{Alignment, Apply, MinIndices, Position, Shim},
    writer::{Branch, DiagramWriter, MergeBranch, WriteBranch},
};

/// A special merge command.
///
/// This merges the trailing minimal indices into column containing the first minimal index. No
/// forks are performed, and the alignment computation will take until account that some of the
/// columns have been removed.
///
/// The merged indices will be deleted regardless of whether they are isolated or not.
///
/// This method is not public since it has some additional requirements:
///
/// 1. It must be applied to every column.
/// 2. The merged columns must be deleted after it is applied.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Merge;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Merge {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        match minimal.pos() {
            Position::Isolated => Align.apply(state, align, span, minimal),
            Position::AfterLast | Position::BeforeFirst => {
                // we know minimal is empty
                fork_impl::<false, false, _, _, _>(state, align, span, [], Branch::Continue)?;
                Ok((1, minimal.is_empty() || span.len() == 1))
            }
            Position::First => fork_impl_generic::<false, true, _, _, _>(
                state,
                align,
                span,
                minimal,
                Branch::MergeStart,
                Branch::ShiftForkLeftMergeStart,
                Branch::ShiftForkRightMergeStart,
            ),
            Position::Inner => {
                state.queue_fill(align.c - align.l);
                state.write_merge_branch(MergeBranch::Join)?;
                Ok((0, true))
            }
            Position::InnerSkipped => {
                state.queue_fill(align.c - align.l);
                state.write_merge_branch(MergeBranch::Cross)?;
                Ok((1, true))
            }
            Position::Last => {
                state.queue_fill(align.c - align.l);
                state.write_merge_branch(MergeBranch::End)?;
                Ok((0, true))
            }
        }
    }
}

/// Either shims a marker, or writes a marker in place of the column while preserving the alignment
/// of the overwritten column.
#[derive(Debug, Clone, Copy)]
pub struct Marker(pub char);

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Marker {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<true, true, _, _, _>(state, align, span, minimal, Branch::Marker(self.0))
    }
}

impl<'a, W: io::Write, B: WriteBranch> Shim<&'a mut DiagramWriter<W, B>> for Marker {
    fn insert(
        self,
        writer: &'a mut DiagramWriter<W, B>,
        gap: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        let leading = gap.c - gap.l;
        writer.queue_fill(leading);
        writer.write_branch(Branch::Marker(self.0))?;
        Ok((leading + 1, 0, true))
    }
}

/// Skip the row, but perform width computations.
#[derive(Debug, Clone, Copy)]
pub struct Skip;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Skip {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<true, true, _, _, _>(state, align, span, minimal, Branch::Marker(' '))
    }
}

/// A shim which acts as an extra column, but still reporting any alignment required by the
/// internal column.
pub struct Extra<'c>(pub &'c mut usize);

impl<'a, 'c, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Extra<'c> {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        // do not branch, but still align and report required alignment
        let ret = fork_impl::<false, true, _, _, _>(state, align, span, minimal, Branch::Continue)?;
        *self.0 = span.last().unwrap().1;
        Ok(ret)
    }
}

impl<'a, 'c, W: io::Write, B: WriteBranch> Shim<&'a mut DiagramWriter<W, B>> for Extra<'c> {
    fn insert(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        // FIXME: this is hacky since it repeats existing manual width computations
        // Maybe it would be best if all of the methods would return `(usize, usize, bool)`
        let l = align.l;

        // create a temporary span representing this column
        let mut span = [((), *self.0)];

        // write the column, modifying the span
        fork_impl::<false, true, _, _, _>(state, align, &mut span, [], Branch::Continue)?;
        // the new column value is exactly this column
        let new_col = span[0].1;

        // the gap is the difference between the new column and the existing one;
        // except that the new column could be smaller
        let gap = 1 + (*self.0).max(new_col) - l;
        *self.0 = span[0].1;

        // ignore the column
        Ok((gap, 0, true))
    }
}

/// Align the column position without branching, and update the alignment correctly.
#[derive(Debug, Clone, Copy)]
pub struct Align;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Align {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, true, _, _, _>(state, align, span, minimal, Branch::Continue)
    }
}

/// Attempt to fork the current column.
#[derive(Debug, Clone, Copy)]
pub struct Fork;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Fork {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, false, _, _, _>(state, align, span, minimal, Branch::Continue)
    }
}

#[inline]
fn fork_impl<const FIXED: bool, const NOBRANCH: bool, V, W: io::Write, B: WriteBranch>(
    writer: &mut DiagramWriter<W, B>,
    col: Alignment,
    span: &mut [(V, usize)],
    minimal: impl IntoIterator<Item = usize>,
    continuation: Branch,
) -> io::Result<(usize, bool)> {
    fork_impl_generic::<FIXED, NOBRANCH, _, _, _>(
        writer,
        col,
        span,
        minimal,
        continuation,
        Branch::ShiftForkLeft,
        Branch::ShiftForkRight,
    )
}

/// Try to expand minimal indices.
///
/// The returned index is the number of extra columns that are required. The returned boolean is `true` if
/// those columns were actually written, and `false` otherwise.
///
/// There are two const parameters.
///
/// - The `FIXED` parameter prevents all writes to the column. If `FIXED` is true, the
///   incoming and outgoing indices will be the same and no branches will be written.
/// - The `NOBRANCH` parameter suppresses branching (so the number of incoming and outgoing
///   branches will be the same) but still allows the index to change.
///
/// Note that the behaviour of `FIXED` implies the behaviour of `NOBRANCH`.
///
/// In either case, alignment computations will still be performed using the set of minimal
/// indices. In order to also suppress alignment computations, explicitly pass an empty list of
/// minimal indices.
#[inline]
fn fork_impl_generic<const FIXED: bool, const NOBRANCH: bool, V, W: io::Write, B: WriteBranch>(
    writer: &mut DiagramWriter<W, B>,
    col: Alignment,
    span: &mut [(V, usize)],
    minimal: impl IntoIterator<Item = usize>,
    continuation: Branch,
    left_branch: impl FnOnce(usize, usize) -> Branch,
    right_branch: impl FnOnce(usize, usize) -> Branch,
) -> io::Result<(usize, bool)> {
    let target = if FIXED { col.c } else { col.clamp() };

    // write preceding whitespace if we don't make it all
    // the way to the beginning
    writer.queue_fill(target.min(col.c) - col.l);

    // The number of required branches we need before we can start branching
    let threshold = col.l.saturating_sub(col.align);

    // The amount of capacity we have for extra branches
    // so we do not exceed the right hand limit.
    let cap = if FIXED || NOBRANCH {
        0
    } else {
        match col.r {
            Some(end) => end - target - 1,
            None => usize::MAX,
        }
    };

    let mut forks = 0; // how many times we forked
    let mut required_forks = 0; // how many times we would have forked if able
    let mut idx = 0; // the current index inside cols

    // whether the previous index was also a minimal index
    // set to `true` to prevent extra increment on the first column
    let mut prev_is_min = true;

    // we do the fine-grained alignment adjustements first, which also computes
    // the new alignment
    for min_idx in minimal {
        while idx < min_idx {
            prev_is_min = false;
            if !(FIXED || NOBRANCH) {
                span[idx].1 += forks;
            }
            idx += 1;
        }

        // if the previous index was a target, the
        // increment has already happened
        if !prev_is_min {
            if threshold <= required_forks && forks < cap {
                forks += 1;
            }
            required_forks += 1;
        }

        // increment the target index
        if !(FIXED || NOBRANCH) {
            span[idx].1 += forks;
        }
        prev_is_min = true;
        idx += 1;

        // prevent an additional increment on the very last column
        if idx < span.len() {
            if threshold <= required_forks && forks < cap {
                forks += 1;
            }
            required_forks += 1;
        }
    }

    // increment any remaining indices
    if !(FIXED || NOBRANCH) {
        while idx < span.len() {
            span[idx].1 += forks;
            idx += 1;
        }
    }

    // apply the global decrement/increment and write the branches
    if target > col.c {
        let increment = target - col.c;

        if !FIXED {
            for (_, c) in span.iter_mut() {
                *c += increment;
            }
        }
        writer.write_branch(right_branch(increment - 1, forks))?;
    } else {
        let decrement = col.c - target;

        if !FIXED && decrement > 0 {
            for (_, c) in span.iter_mut() {
                *c -= decrement;
            }
        }

        // work out the correct drawing
        let br = if decrement == 0 {
            forks
                .checked_sub(1)
                .map(Branch::ForkRight)
                .unwrap_or(continuation)
        } else if decrement < forks {
            Branch::ForkMiddle(decrement - 1, forks - decrement - 1)
        } else if decrement == forks {
            Branch::ForkLeft(forks - 1)
        } else {
            left_branch(decrement - forks - 1, forks)
        };

        writer.write_branch(br)?;
    };

    Ok((required_forks + 1, required_forks == forks))
}
