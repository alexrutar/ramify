#[cfg(test)]
mod tests;

use std::io;

use crate::{
    columns::{Alignment, Apply, ColumnIndexIter, Gap, Shim},
    writer::{Branch, DiagramWriter, WriteBranch},
};

/// Compact much as possible, ignoring minimal indices.
///
/// This is the opposite of [`Isolate`].
#[derive(Debug, Clone, Copy)]
pub struct Compact;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Compact {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        _minimal: ColumnIndexIter<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, false, _, _, _>(state, align, span, [], Branch::Continue)
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
        minimal: ColumnIndexIter<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<true, true, _, _, _>(state, align, span, minimal, Branch::Marker(self.0))
    }
}

impl<'a, W: io::Write, B: WriteBranch> Shim<&'a mut DiagramWriter<W, B>> for Marker {
    fn insert(self, writer: &'a mut DiagramWriter<W, B>, gap: Gap) -> Result<usize, Self::Error> {
        let leading = gap.c - gap.l;
        writer.queue_blank(leading);
        writer.write_branch(Branch::Marker(self.0))?;
        Ok(leading + 1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MarkerGreedy(pub char);

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for MarkerGreedy {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: ColumnIndexIter<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        if minimal.is_empty() {
            fork_impl::<true, true, _, _, _>(state, align, span, [], Branch::Marker(self.0))
        } else {
            fork_impl::<true, true, _, _, _>(
                state,
                align,
                span,
                0..span.len(),
                Branch::Marker(self.0),
            )
        }
    }
}

impl<'a, W: io::Write, B: WriteBranch> Shim<&'a mut DiagramWriter<W, B>> for MarkerGreedy {
    fn insert(self, writer: &'a mut DiagramWriter<W, B>, gap: Gap) -> Result<usize, Self::Error> {
        Marker(self.0).insert(writer, gap)
    }
}

/// Preserve the current column position, but still update the alignment correctly.
#[derive(Debug, Clone, Copy)]
pub struct Preserve;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Preserve {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: ColumnIndexIter<'_>,
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
        minimal: ColumnIndexIter<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, false, _, _, _>(state, align, span, minimal, Branch::Continue)
    }
}

/// Attempt to isolate every column in column blocks which contain at least one minimal element.
///
/// This applies [`Compact`] if there are no minimal indices, and [`Isolate`] otherwise.
#[derive(Debug, Clone, Copy)]
pub struct ForkGreedy;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for ForkGreedy {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: ColumnIndexIter<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        if minimal.is_empty() {
            Compact.apply(state, align, span, minimal)
        } else {
            Isolate.apply(state, align, span, minimal)
        }
    }
}

/// Attempt to isolate every column, regardless of minimality.
///
/// This is the opposite of [`Compact`].
#[derive(Debug, Clone, Copy)]
pub struct Isolate;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Isolate {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        _: ColumnIndexIter<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, false, _, _, _>(state, align, span, 0..span.len(), Branch::Continue)
    }
}

/// Try to expand minimal indices.
///
/// The returned index is the number of extra columns that are required. The boolean is `true` if
/// those columns were actually written, and `false` otherwise.
///
/// If `FIXED` is false, the column will be modified. Otherwise, alignment computations will still
/// be performed, but the column indices will be unchanged.
///
/// This is a generic implementation designed to be inlined for optimization since setting `FIXED =
/// false` or providing an empty iterator for `minimal` causes substantial simplification to the
/// algorithm.
#[inline]
fn fork_impl<const FIXED: bool, const NOBRANCH: bool, V, W: io::Write, B: WriteBranch>(
    writer: &mut DiagramWriter<W, B>,
    col: Alignment,
    span: &mut [(V, usize)],
    minimal: impl IntoIterator<Item = usize>,
    continuation: Branch,
) -> io::Result<(usize, bool)> {
    let target = if FIXED { col.c } else { col.clamp() };

    // write preceding whitespace if we don't make it all
    // the way to the beginning
    writer.queue_blank(target.min(col.c) - col.l);

    // The number of required branches we need before we can start branching
    let threshold = col.l.saturating_sub(col.align);

    // The amount of capacity we have for extra branches
    // so we do not exceed the right hand limit.
    let cap = if NOBRANCH {
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

    // NOTE: If !FIXED, then `forks` is never modified and `target = col.c`
    // so none of the writes to `span` do anything. However, manually suppressing
    // each block simplifies codegen.

    // whether the previous index was also a minimal index
    // set to `true` to prevent extra increment on the first column
    let mut prev_is_min = true;

    // we do the fine-grained alignment adjustements first, which also computes
    // the new alignment
    for min_idx in minimal {
        while idx < min_idx {
            prev_is_min = false;
            if !FIXED {
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
        if !FIXED {
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
    if !FIXED {
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
        writer.write_branch(Branch::ShiftForkRight(increment - 1, forks))?;
    } else {
        let decrement = col.c - target;

        if !FIXED && decrement > 0 {
            for (_, c) in span.iter_mut() {
                *c -= decrement;
            }
        }

        // work out the correct drawing
        let br = if decrement == 0 {
            match forks.checked_sub(1) {
                None => continuation,
                Some(n) => Branch::ForkRight(n),
            }
        } else if decrement < forks {
            Branch::ForkMiddle(decrement - 1, forks - decrement - 1)
        } else if decrement == forks {
            // forks > 0 since align > 0
            Branch::ForkLeft(forks - 1)
        } else {
            // align > forks
            Branch::ShiftForkLeft(decrement - forks - 1, forks)
        };

        writer.write_branch(br)?;
    };

    Ok((required_forks, required_forks == forks))
}
