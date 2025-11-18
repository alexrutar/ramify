#[cfg(test)]
mod tests;

use std::{
    io,
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

use crate::writer::{Branch, Writer};

/// A half-open [`usize`] range; essentially, one of `..`, `a..`, `..b`, or `a..b`.
pub trait HalfOpen {
    /// The inclusive start index.
    fn start(&self) -> usize;

    /// The end index, or `None` of unbounded.
    fn end(&self) -> Option<usize>;
}

impl HalfOpen for RangeFull {
    fn start(&self) -> usize {
        0
    }

    fn end(&self) -> Option<usize> {
        None
    }
}

impl HalfOpen for RangeFrom<usize> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> Option<usize> {
        None
    }
}

impl HalfOpen for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> Option<usize> {
        Some(self.end)
    }
}

impl HalfOpen for RangeTo<usize> {
    fn start(&self) -> usize {
        0
    }

    fn end(&self) -> Option<usize> {
        Some(self.end)
    }
}

/// two options: use this value, or max(this value, current number of cols) to
/// avoid jitter
pub fn required_width<V>(cols: &[(V, usize)], min_index: usize) -> usize {
    // count the number of distinct column values plus the number of chars needed to fork
    let mut num_cols = 0;
    let mut idx = 0;
    while idx < cols.len() {
        num_cols += 1;
        let cur_col = cols[idx].1;
        while idx < cols.len() && cols[idx].1 == cur_col {
            idx += 1;
        }
    }
    let Range { start: l, end: r } = column_range(cols, min_index);
    let extra_fork_space = if l + 1 == r {
        0
    } else if min_index == l || min_index + 1 == r {
        1
    } else {
        2
    };
    num_cols + extra_fork_space
}

/// Write the marker character.
///
/// Since the marker position cannot move, write blanks before the marker if necessary.
pub fn marker<W: io::Write>(
    writer: &mut Writer<W>,
    marker: char,
    offset: usize,
    marker_col: usize,
) -> io::Result<usize> {
    if marker_col >= offset {
        writer.write_branch(Branch::Blank(marker_col - offset))?;
        writer.write_vertex_marker(marker)?;
        Ok(marker_col + 1)
    } else {
        writer.write_vertex_marker(marker)?;
        // propagate the offset
        Ok((marker_col + 1).max(offset + 1))
    }
}

/// Write the marker character and also do computations to adjust the returned offset to try to
/// make space for the next marker
pub fn mark_and_prepare<V, W: io::Write>(
    writer: &mut Writer<W>,
    cols: &[(V, usize)],
    marker: char,
    offset: usize,
    min_index: usize,
) -> io::Result<usize> {
    debug_assert!(min_index < cols.len());
    let Range { start: l, end: r } = column_range(cols, min_index);

    let col = cols[min_index].1;

    writer.write_branch(Branch::Blank(col.saturating_sub(offset)))?;
    writer.write_vertex_marker(marker)?;

    // the number of columns we require to perform the fork later
    let required_fork_space = if l + 1 == r {
        1
    } else if min_index == l || min_index + 1 == r {
        2
    } else {
        3
    };
    Ok((col + 1).max(offset + required_fork_space))
}

/// Given a set of columns and a minimal index valid for the set of columns,
/// compute the range of indices which match the provided column.
pub fn column_range<V>(cols: &[(V, usize)], idx: usize) -> Range<usize> {
    debug_assert!(idx < cols.len());
    let min_index_col = cols[idx].1;
    let mut start = idx;

    while start > 0 && cols[start - 1].1 == min_index_col {
        start -= 1;
    }

    let mut end = idx + 1;

    while end < cols.len() && cols[end].1 == min_index_col {
        end += 1;
    }
    Range { start, end }
}

/// Attempt to modify a set of columns so that they all land within the provided bounds.
///
/// The `bounds` argument can be one of the ranges `..`, `a..`, or `a..b`, or anything else which
/// implements the [`HalfOpen`] trait.
///
/// This returns the end index of the range required to satisfy the provided alignment.
/// If the alignment was satisfied, this is just the last column plus one. When chaining alignments
/// together, this index should be used as the start bound for the subsequent alignment.
pub fn align<V, W: io::Write>(
    writer: &mut Writer<W>,
    cols: &mut [(V, usize)],
    bounds: impl HalfOpen,
) -> io::Result<usize> {
    let mut start = bounds.start();
    let mut idx = 0;

    while idx < cols.len() {
        let cur_col = cols[idx].1;
        if cur_col >= start {
            let diff = cur_col - start;

            writer.write_branch(Branch::shift_left(diff))?;

            start = cur_col + 1;
            if diff >= 1 {
                // decrement all of the subsequent elements, as long as the value is the same
                let new_col = cur_col - diff;
                cols[idx].1 = new_col;
                idx += 1;
                while idx < cols.len() && cols[idx].1 == cur_col {
                    cols[idx].1 = new_col;
                    idx += 1;
                }
            } else {
                // skip elements of the same value
                while idx < cols.len() && cols[idx].1 == cur_col {
                    idx += 1;
                }
            }
        } else {
            // the amount of right shift we would like to make, if we are able
            let required_shift = start - cur_col;

            let prev_idx = idx;

            // find the index range for the current column block
            // let mut end = idx;
            while idx < cols.len() && cols[idx].1 == cur_col {
                idx += 1;
            }

            // compute the amount of shift we are actually permitted to make
            let allowed_shift = if idx < cols.len() {
                // next element
                let next_col = cols[idx].1;
                // we need cur_col + allowed_shift < next_col
                required_shift.min(next_col - cur_col - 1)
            } else if let Some(next_col) = bounds.end() {
                // bounded case; add a check in case the end bound is small
                required_shift.min(next_col.saturating_sub(cur_col + 1))
            } else {
                // unbounded case
                required_shift
            };

            // the new column value
            let new_col = cur_col + allowed_shift;

            writer.write_branch(Branch::shift_right(allowed_shift))?;
            for (_, c) in cols[prev_idx..idx].iter_mut() {
                *c = new_col;
            }

            // if we were not able to make the amount of shift that we wanted,
            // request an extra space to avoid jitter
            if required_shift > allowed_shift {
                start += 1;
            }

            start = start.max(new_col + 1);
        }
    }

    Ok(start)
}

/// Resolve forks and alignment.
///
/// The prototypical situation we have to handle is the following.
/// ```txt
/// 0
/// ├┬╮
/// │1│
/// ├╮2 <- printing this row
/// 3││
///  4│
///   5
/// ```
/// We just wrote the marker for `2`, and in the previous line we write the marker `1`, but it had
/// no children so now there is a hole.
///
/// The initial column state is
/// ```txt
/// [(3, 0), (4, 0), (*2,2)]
/// ```
/// We see that `(*2,2)` does not require forking to resolve, so we substitute:
/// ```txt
/// [(*3,0), (4, 0), (5, 2)]
/// ```
/// Now we try to resolve forks in the initial segment, and moreover we can use the free column 1
/// to do this. This is the `free_col` boolean argument which is passed to this method.
///
/// In contrast, in a situation like
/// ```txt
/// 0
/// ├┬╮
/// │1│
/// ├╮│ <- printing this row
/// 2││
///  3│
///   4
/// ```
/// this is no longer the case: we cannot fork immediately before vertex `2` because vertex `1` is
/// still occupying the position, so we need to wait one extra row to fork.
pub fn fork_align<V, W: io::Write, const FORK: bool>(
    writer: &mut Writer<W>,
    cols: &mut [(V, usize)],
    min_index: usize,
    bounds: impl HalfOpen,
) -> io::Result<usize> {
    debug_assert!(min_index < cols.len());
    let Range { start: l, end: r } = column_range(cols, min_index);
    let mut offset = bounds.start();

    if l + 1 == r {
        // fork is not required since the minimal index is isolated
        return align(writer, cols, bounds);
    }

    // align up to the starting index, and update the column
    offset = align(writer, &mut cols[..l], offset..)?;

    // perform the fork, but do not exceed either the end bound or the next char
    let fork_limit = match cols.get(r) {
        Some(end) => Some(end.1),
        None => bounds.end(),
    };

    offset = match fork_limit {
        Some(end) => fork_exact::<_, _, FORK>(writer, &mut cols[l..r], min_index - l, offset..end)?,
        None => fork_exact::<_, _, FORK>(writer, &mut cols[l..r], min_index - l, offset..)?,
    };

    match bounds.end() {
        Some(end) => align(writer, &mut cols[r..], offset..end),
        None => align(writer, &mut cols[r..], offset..),
    }
}

/// Perform a fork, where the fork corresponds exactly to the provided columns.
pub fn fork_exact<V, W: io::Write, const FORK: bool>(
    writer: &mut Writer<W>,
    cols: &mut [(V, usize)],
    min_index: usize,
    bounds: impl HalfOpen,
) -> io::Result<usize> {
    debug_assert!(min_index < cols.len());
    let mut offset = bounds.start();

    if min_index == 0 || min_index + 1 == cols.len() {
        // boundary fork
        let cur_col = cols[min_index].1;
        let space_on_left = cur_col - offset;

        if space_on_left == 0 {
            let can_fork_right = match bounds.end() {
                Some(bd) => bd > cur_col + 1,
                None => true,
            };

            if can_fork_right {
                if FORK {
                    writer.write_branch(Branch::ForkRight(0))?;
                    if min_index == 0 {
                        // the new minimal element follows the left fork, so all of the other
                        // elements follow the right fork
                        for (_, c) in cols[1..].iter_mut() {
                            *c += 1;
                        }
                    } else {
                        // the new minimal element follows the right fork
                        cols[min_index].1 = cur_col + 1;
                    }
                } else if min_index == 0 {
                    writer.write_branch(Branch::Continue)?;
                    writer.write_branch(Branch::Blank(1))?;
                } else {
                    writer.write_branch(Branch::ShiftForkRight(0, 0))?;
                    for (_, c) in cols {
                        *c += 1;
                    }
                }
            } else {
                writer.write_branch(Branch::Continue)?;
            }

            // either we fork right or we fail; either way, we request an extra space
            offset = cur_col + 2;
        } else {
            if FORK {
                if space_on_left == 1 {
                    writer.write_branch(Branch::ForkLeft(0))?;
                } else {
                    writer.write_branch(Branch::ShiftForkLeft(space_on_left - 2, 1))?;
                }

                if min_index == 0 {
                    cols[0].1 = cur_col - space_on_left;
                    for (_, c) in cols[1..].iter_mut() {
                        *c = *c + 1 - space_on_left;
                    }
                } else {
                    for (_, c) in cols[0..min_index].iter_mut() {
                        *c -= space_on_left;
                    }
                    cols[min_index].1 = cols[min_index].1 + 1 - space_on_left;
                }
            } else {
                // align with the branch that will write the next vertex
                if min_index == 0 {
                    writer.write_branch(Branch::ShiftForkLeft(space_on_left - 1, 0))?;
                    for (_, c) in cols {
                        *c -= space_on_left;
                    }
                } else if space_on_left == 1 {
                    writer.write_branch(Branch::Blank(1))?;
                    writer.write_branch(Branch::Continue)?;
                } else {
                    writer.write_branch(Branch::Blank(1))?;
                    writer.write_branch(Branch::ShiftForkLeft(space_on_left - 2, 0))?;
                    for (_, c) in cols {
                        *c -= space_on_left - 1;
                    }
                }
            }

            // we forked left successfully, so we don't need to request an extra space
            offset = cur_col + 1;
        }
    } else {
        // interior fork, which has size 3
        const EXTRA: usize = 2;

        let cur_col = cols[min_index].1;
        let space_on_left = cur_col - offset;
        let space_on_right = match bounds.end() {
            Some(bd) => bd.saturating_sub(cur_col + 1),
            None => EXTRA,
        };

        if space_on_left + space_on_right >= EXTRA {
            if FORK {
                // write the fork or shift and update the previous column
                if space_on_left > EXTRA {
                    writer.write_branch(Branch::ShiftForkLeft(space_on_left - EXTRA - 1, EXTRA))?;
                    offset = cur_col + 1;
                } else {
                    writer.write_branch(Branch::fork(space_on_left, EXTRA - space_on_left))?;
                    offset = cur_col + EXTRA - space_on_left + 1;
                };

                // update the column values
                for (_, c) in cols[..min_index].iter_mut() {
                    *c -= space_on_left;
                }
                cols[min_index].1 = cur_col - space_on_left + 1;
                for (_, c) in cols[min_index + 1..].iter_mut() {
                    *c = *c - space_on_left + 2;
                }
            } else if space_on_left == 0 {
                writer.write_branch(Branch::ShiftForkRight(0, 0))?;
                writer.write_branch(Branch::Blank(1))?;

                for (_, c) in cols {
                    *c += 1;
                }
                offset = cur_col + 3;
            } else if space_on_left == 1 {
                writer.write_branch(Branch::Blank(1))?;
                writer.write_branch(Branch::Continue)?;
                writer.write_branch(Branch::Blank(1))?;

                offset = cur_col + 2;
            } else {
                writer.write_branch(Branch::Blank(1))?;
                writer.write_branch(Branch::ShiftForkLeft(space_on_left - 2, 0))?;

                for (_, c) in cols {
                    *c -= space_on_left - 1;
                }
                offset = cur_col + 1;
            }
        } else {
            if space_on_left > 0 {
                writer.write_branch(Branch::Blank(1))?;
            }
            writer.write_branch(Branch::Continue)?;
            if space_on_right > 0 {
                writer.write_branch(Branch::Blank(1))?;
            }
            offset = cur_col + 1 + EXTRA - space_on_left;
        }
    };
    Ok(offset)
}
