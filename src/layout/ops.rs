#[cfg(test)]
pub(crate) mod tests;

use crate::{
    columns::{Alignment, Apply, MinIndices, Position, Shim},
    writer::{Branch, DiagramWrite, MergeBranch},
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

impl<'a, W: DiagramWrite> Apply<&'a mut W> for Merge {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        match minimal.pos() {
            Position::Isolated => Align.apply(state, align, span, minimal),
            Position::AfterLast | Position::BeforeFirst => {
                fork_impl_suppressed::<false, _, _>(state, align, span, minimal, Branch::Continue)
            }
            Position::First => fork_impl_generic::<false, true, _, _>(
                state,
                align,
                span,
                minimal,
                Branch::MergeStart,
                Branch::ShiftForkLeftMergeStart,
                Branch::ShiftForkRightMergeStart,
            ),
            Position::Inner => {
                state.write_merge_branch(align.l, align.c - align.l, MergeBranch::Join)?;
                Ok((0, true))
            }
            Position::InnerSkipped => {
                state.write_merge_branch(align.l, align.c - align.l, MergeBranch::Cross)?;
                Ok((1, true))
            }
            Position::Last => {
                state.write_merge_branch(align.l, align.c - align.l, MergeBranch::End)?;
                Ok((0, true))
            }
        }
    }
}

/// Either shims a marker, or writes a marker in place of the column while preserving the alignment
/// of the overwritten column.
#[derive(Debug, Clone, Copy)]
pub struct DelayedMarker(pub char);

impl<'a, W: DiagramWrite> Apply<&'a mut W> for DelayedMarker {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl_suppressed::<false, _, _>(state, align, span, minimal, Branch::Marker(self.0))
    }
}

impl<'a, W: DiagramWrite> Shim<&'a mut W> for DelayedMarker {
    fn insert(
        self,
        writer: &'a mut W,
        gap: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        let leading = gap.c - gap.l;
        writer.write_branch(gap.l, leading, Branch::Marker(self.0))?;
        Ok((leading + 1, 0, true))
    }
}

/// Either shims a marker, or writes a marker in place of the column while preserving the alignment
/// of the overwritten column.
#[derive(Debug, Clone, Copy)]
pub struct Marker(pub char);

impl<'a, W: DiagramWrite> Apply<&'a mut W> for Marker {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<true, true, _, _>(state, align, span, minimal, Branch::Marker(self.0))
    }
}

impl<'a, W: DiagramWrite> Shim<&'a mut W> for Marker {
    fn insert(
        self,
        writer: &'a mut W,
        gap: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        let leading = gap.c - gap.l;
        writer.write_branch(gap.l, leading, Branch::Marker(self.0))?;
        Ok((leading + 1, 0, true))
    }
}

/// Skip the row and suppress width computations.
#[derive(Debug, Clone, Copy)]
pub struct Skip;

impl<'a, W: DiagramWrite> Apply<&'a mut W> for Skip {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl_suppressed::<true, _, _>(state, align, span, minimal, Branch::Marker(' '))
    }
}

/// A shim which acts as an extra column, but still reporting any alignment required by the
/// internal column.
pub struct DelayedFork<'c>(pub &'c mut usize);

impl<'a, 'c, W: DiagramWrite> Apply<&'a mut W> for DelayedFork<'c> {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        let ret =
            fork_impl_suppressed::<false, _, _>(state, align, span, minimal, Branch::Continue)?;
        *self.0 = span.last().unwrap().1;
        Ok(ret)
    }
}

impl<'a, 'c, W: DiagramWrite> Shim<&'a mut W> for DelayedFork<'c> {
    fn insert(
        self,
        state: &'a mut W,
        align: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        // FIXME: this is hacky since it repeats existing manual width computations
        // Maybe it would be best if all of the methods would return `(usize, usize, bool)`
        let l = align.l;

        // create a temporary span representing this column
        let mut span = [((), *self.0)];

        // write the column, modifying the span
        fork_impl::<false, true, _, _>(state, align, &mut span, [], Branch::Continue)?;
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

impl<'a, W: DiagramWrite> Apply<&'a mut W> for Align {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, true, _, _>(state, align, span, minimal, Branch::Continue)
    }
}

/// Attempt to fork the current column.
#[derive(Debug, Clone, Copy)]
pub struct Fork;

impl<'a, W: DiagramWrite> Apply<&'a mut W> for Fork {
    type Error = W::Error;

    fn apply<V>(
        self,
        state: &'a mut W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, false, _, _>(state, align, span, minimal, Branch::Continue)
    }
}

/// Write a row, ignoring the minimal indices but still reporting if the column is isolated or not.
#[inline]
fn fork_impl_suppressed<const FIXED: bool, V, W: DiagramWrite>(
    writer: &mut W,
    col: Alignment,
    span: &mut [(V, usize)],
    minimal: MinIndices<'_>,
    continuation: Branch,
) -> Result<(usize, bool), W::Error> {
    fork_impl::<FIXED, true, _, _>(writer, col, span, [], continuation)?;
    Ok((1, minimal.is_empty() || span.len() == 1))
}

#[inline]
fn fork_impl<const FIXED: bool, const NOBRANCH: bool, V, W: DiagramWrite>(
    writer: &mut W,
    col: Alignment,
    span: &mut [(V, usize)],
    minimal: impl IntoIterator<Item = usize>,
    continuation: Branch,
) -> Result<(usize, bool), W::Error> {
    fork_impl_generic::<FIXED, NOBRANCH, _, _>(
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
fn fork_impl_generic<const FIXED: bool, const NOBRANCH: bool, V, W: DiagramWrite>(
    writer: &mut W,
    col: Alignment,
    span: &mut [(V, usize)],
    minimal: impl IntoIterator<Item = usize>,
    continuation: Branch,
    left_branch: impl FnOnce(usize, usize) -> Branch,
    right_branch: impl FnOnce(usize, usize) -> Branch,
) -> Result<(usize, bool), W::Error> {
    let target = if FIXED { col.c } else { col.clamp() };
    let leading = target.min(col.c) - col.l;

    // write preceding whitespace if we don't make it all
    // the way to the beginning
    // writer.write_fill(target.min(col.c) - col.l)?;

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
        writer.write_branch(col.l, leading, right_branch(increment - 1, forks))?;
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

        writer.write_branch(col.l, leading, br)?;
    };

    Ok((required_forks + 1, required_forks == forks))
}
