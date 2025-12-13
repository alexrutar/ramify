#[cfg(test)]
mod tests;

use std::{
    cmp::Ordering,
    slice::{ChunkByMut, Iter},
};

/// An operation which can be applied to a column.
pub trait Apply<W> {
    type Error;

    /// Apply the map to a column.
    fn apply<V>(
        self,
        state: W,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error>;
}

/// An operation which can be applied to a column or between columns.
pub trait Shim<W>: Apply<W> {
    /// Insert into the provided gap.
    fn insert(self, state: W, gap: Gap) -> Result<usize, Self::Error>;
}

/// The alignment of a column.
///
/// Invariants:
///
/// 1. `self.l <= self.c`
/// 2. `self.r.is_none_or(|x| self.c < x)`
#[derive(Debug)]
pub struct Alignment {
    /// The lower limit for legal writes.
    pub l: usize,
    /// The upper limit for legal writes.
    pub r: Option<usize>,
    /// The targeted left alignment. This is what `self.c` would be
    /// when all columns are fully expanded and there are no gaps.
    pub align: usize,
    /// The original column value for this block
    pub c: usize,
}

impl Alignment {
    /// Clamp the alignment to land in the range `l..r` (or `l..` if `r` is `None`).
    pub fn clamp(&self) -> usize {
        let t = self.l.max(self.align);
        match self.r {
            None => t,
            Some(mx) => t.min(mx - 1), // Inv 5
        }
    }
}

/// A gap between two columns.
#[allow(unused)]
pub struct Gap {
    /// The lower limit for legal writes.
    pub l: usize,
    /// The shim column.
    pub c: usize,
    /// The upper limit for legal writes.
    pub r: Option<usize>,
    /// The targeted left alignment.
    pub align: usize,
}

/// The minimal indices corresponding to a given column.
#[derive(Debug)]
pub struct MinIndices<'a> {
    first: Option<usize>,
    rest: Iter<'a, usize>,
    diff: usize,
    lt_first: bool,
    geq_last: bool,
}

pub enum Position {
    /// An index before the first minimal index.
    BeforeFirst,
    /// The only minimal index.
    Isolated,
    /// The first minimal index, which is not also the last.
    First,
    /// A minimal index which is not the first or the last.
    Inner,
    /// An index which is between the first and last minimal indices, but is not minimal.
    InnerSkipped,
    /// The last minimal index, which is not also the first.
    Last,
    /// An index after the last minimal index.
    AfterLast,
}

impl<'a> MinIndices<'a> {
    pub fn new(
        first: Option<usize>,
        rest: &'a [usize],
        diff: usize,
        lt_first: bool,
        geq_last: bool,
    ) -> Self {
        Self {
            first,
            rest: rest.iter(),
            diff,
            lt_first,
            geq_last,
        }
    }

    /// Return the position of this minimal index relative to the other minimal indices.
    pub fn pos(&self) -> Position {
        if self.lt_first {
            Position::BeforeFirst
        } else if self.first.is_some() {
            if self.geq_last {
                Position::Isolated
            } else {
                Position::First
            }
        } else {
            match (self.rest.as_slice().is_empty(), self.geq_last) {
                (false, false) => Position::Inner,
                (true, false) => Position::InnerSkipped,
                (false, true) => Position::Last,
                (true, true) => Position::AfterLast,
            }
        }
    }
}

impl MinIndices<'static> {
    pub fn empty(lt_first: bool, geq_last: bool) -> Self {
        Self {
            first: None,
            rest: [].iter(),
            diff: 0,
            lt_first,
            geq_last,
        }
    }
}

impl<'a> Iterator for MinIndices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.first.take() {
            Some(e) => e - self.diff,
            None => self.rest.next()? - self.diff,
        })
    }
}

#[derive(Debug)]
pub struct Status {
    /// Whether the minimal vertices are all isolated.
    pub isolated: bool,
    /// The largest column which contains characters.
    pub width: usize,
    /// The number of columns if the minimal indices are isolated.
    pub target_width: usize,
}

impl Status {
    /// The number of columns required.
    pub fn reserved_width(&self) -> usize {
        self.width.max(self.target_width)
    }
}

/// A stateful iterator of columns.
pub struct ColumnsMut<'a, V> {
    inner: RawColumnIter<'a, V>,
    l: usize,
    isolated: bool,
    align: usize,
    // FIXME: it would be better if `ColumnsMut`, `Shimmed`, and a new `Bounded`
    // were all implementors of some trait, and `Shimmed` and `Bounded` were
    // generic over the trait
    bound: Option<usize>,
}

impl<'a, V> ColumnsMut<'a, V> {
    /// Initialize a new left-aligned mutable column iterator.
    pub fn new(active: &'a mut [(V, usize)], minimal: Option<(usize, &'a [usize])>) -> Self {
        Self::with_alignment(active, minimal, 0)
    }

    pub fn with_bound(mut self, bound: usize) -> Self {
        self.bound = Some(bound);
        self
    }

    /// Initialize a mutable column iterator with an initial target alignment.
    pub fn with_alignment(
        active: &'a mut [(V, usize)],
        minimal: Option<(usize, &'a [usize])>,
        align: usize,
    ) -> Self {
        let inner = RawColumnIter::init(active, minimal);
        Self {
            inner,
            l: 0,
            isolated: true,
            align,
            bound: None,
        }
    }

    pub fn status(&self) -> Status {
        Status {
            isolated: self.isolated,
            width: self.l,
            target_width: self.align,
        }
    }

    /// Peek the value of the next column.
    pub fn peek_col(&self) -> Option<usize> {
        self.inner.peek_col()
    }

    fn apply_impl<F, T>(
        &mut self,
        f: F,
        state: T,
        (c, span, r, minimal): (usize, &mut [(V, usize)], Option<usize>, MinIndices<'_>),
    ) -> Result<Option<usize>, F::Error>
    where
        F: Apply<T>,
    {
        let col = Alignment {
            l: self.l,
            align: self.align,
            r: r.or(self.bound),
            c,
        };

        let (extra, incomplete) = f.apply(state, col, span, minimal)?;

        self.isolated &= incomplete;

        self.align += extra;
        let new_c = span.last().unwrap().1;
        self.l = c.max(new_c) + 1;

        Ok(r)
    }

    /// Apply a closure to the next column and increment the internal state.
    ///
    /// This returns the column index of the next column, if any.
    ///
    /// The method returns `None` if there are no more columns. If a previous call to `apply`
    /// returned `None`, the closure will not called.
    pub fn apply<F, T>(&mut self, f: F, state: T) -> Result<Option<usize>, F::Error>
    where
        F: Apply<T>,
    {
        let Some(nxt) = self.inner.next() else {
            return Ok(None);
        };

        self.apply_impl(f, state, nxt)
    }

    pub fn with_shim<S>(self, shim: (usize, S)) -> Shimmed<'a, V, S> {
        Shimmed {
            cols: self,
            shim: Some(shim),
        }
    }
}

/// A stateful iterator of columns which also inserts shim at a specific column.
///
/// A shim contains a column, along with a callback to apply if that column is reached.
pub struct Shimmed<'a, V, S> {
    cols: ColumnsMut<'a, V>,
    shim: Option<(usize, S)>, // `None` if the shim no longer applies
}

impl<'a, V, S> Shimmed<'a, V, S> {
    pub fn cols(&self) -> &ColumnsMut<'a, V> {
        &self.cols
    }

    /// Apply the shim with the provided upper bound.
    #[inline]
    fn apply_shim<T>(
        &mut self,
        shim: S,
        state: T,
        c: usize,
        r: Option<usize>,
    ) -> Result<Option<usize>, S::Error>
    where
        S: Shim<T>,
    {
        let gap = Gap {
            l: self.cols.l,
            c,
            r,
            align: self.cols.align,
        };

        let shimmed = shim.insert(state, gap)?;

        self.cols.l += shimmed;
        Ok(r)
    }

    /// Apply the given function, also taking into account the shim.
    ///
    /// If the shim matches the column, it is applied instead of the closure.
    ///
    /// This returns the column index of the next column, if any. The next column
    /// might be the shim.
    pub fn apply<E, F, T>(&mut self, f: F, state: T) -> Result<Option<usize>, E>
    where
        S: Shim<T, Error = E>,
        F: Apply<T, Error = E>,
    {
        match self.shim.take() {
            // shim does not apply since we are past it
            Some((col, _)) if col < self.cols.l => self.cols.apply(f, state),
            None => self.cols.apply(f, state),
            // shim might still apply
            Some((col, shim)) => {
                // now self.cols.l < col

                // peek the next column value
                match self.cols.peek_col() {
                    Some(c) => {
                        match col.cmp(&c) {
                            Ordering::Less => {
                                // insert the shim
                                self.apply_shim(shim, state, col, Some(c))
                            }
                            Ordering::Equal => {
                                // apply the shim instead of the closure
                                self.cols.apply(shim, state)
                            }
                            Ordering::Greater => {
                                // the shim might apply in the future; use the column to avoid
                                // writing into the shim column
                                self.shim = Some((col, shim));

                                let (c, span, r, minimal) = self
                                    .cols
                                    .inner
                                    .next()
                                    .expect("Already checked that there is a new column");

                                let bound = match r {
                                    Some(bd) => bd.min(col),
                                    None => col,
                                };

                                // the correct bound is returned here because we already overwrote
                                // it
                                self.cols
                                    .apply_impl(f, state, (c, span, Some(bound), minimal))
                            }
                        }
                    }
                    None => {
                        // we reached the end without applying the shim,
                        // so we just apply it now
                        self.apply_shim(shim, state, col, None)
                    }
                }
            }
        }
    }
}

/// The raw iterator over the chunks
type ChunksIter<'a, V> = ChunkByMut<'a, (V, usize), fn(&(V, usize), &(V, usize)) -> bool>;

/// A raw iterator over columns.
///
/// A column is a block of active vertices for which the column index is identical for all of the
/// vertices.
struct RawColumnIter<'a, V> {
    remaining: ChunksIter<'a, V>,
    // the first column index of remaining, or none if remaining is empty
    peeked: Option<&'a mut [(V, usize)]>,
    first_minimal: Option<usize>,
    // the minimal indices
    minimal: &'a [usize],
    // the current position inside the original set of columns
    pos: usize,
}

impl<'a, V> RawColumnIter<'a, V> {
    pub fn init(active: &'a mut [(V, usize)], minimal: Option<(usize, &'a [usize])>) -> Self {
        // type annotation to coerce `FnOnce` to `fn`
        let mut remaining: ChunksIter<'a, V> = active.chunk_by_mut(|a, b| a.1 == b.1);
        let peeked = remaining.next();

        let (first_minimal, minimal) = match minimal {
            Some((fst, rest)) => (Some(fst), rest),
            None => (None, &[][..]),
        };

        Self {
            remaining,
            first_minimal,
            minimal,
            peeked,
            pos: 0,
        }
    }

    /// Returns the value of the next column.
    pub fn peek_col(&self) -> Option<usize> {
        self.peeked.as_ref().map(|n| n.first().unwrap().1)
    }
}

impl<'a, V> Iterator for RawColumnIter<'a, V> {
    type Item = (usize, &'a mut [(V, usize)], Option<usize>, MinIndices<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        // swap the peeked element with the new peeked element
        let current = self.peeked.take()?;
        self.peeked = self.remaining.next();

        // SAFETY: ChunksByMut slices are always be non-empty
        let c = unsafe { current.get_unchecked(0).1 };
        let r = self
            .peeked
            .as_ref()
            .map(|p| unsafe { p.get_unchecked(0).1 });

        // increment the position threshold
        let diff = self.pos;
        self.pos += current.len();

        // get the column index iterator
        let it = if self.first_minimal.is_some_and(|e| e >= self.pos) {
            // we know there is another minimal index later
            MinIndices::empty(true, true)
        } else {
            let first = self.first_minimal.take();
            let mut min_idx = 0;
            while min_idx < self.minimal.len() && self.minimal[min_idx] < self.pos {
                min_idx += 1;
            }
            // check if there are more indices
            let last = min_idx == self.minimal.len();
            let rest = self.minimal.split_off(..min_idx).unwrap();
            MinIndices::new(first, rest, diff, false, last)
        };

        Some((c, current, r, it))
    }
}
