//! Operations which compact or preserve alignment.

use std::io;

use super::{Continue, Marker, fork_impl};
use crate::{
    columns::{Alignment, Apply, MinIndices, Shim},
    writer::{Branch, DiagramWriter, WriteBranch},
};

/// Compact much as possible, ignoring minimal indices but still reporting isolation correctly.
#[derive(Debug, Clone, Copy)]
pub struct Compact;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for Compact {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        fork_impl::<false, false, _, _, _>(state, align, span, [], Branch::Continue)?;
        Ok((1, minimal.is_empty() || span.len() == 1))
    }
}

/// Skip the row, but perform width computations.
#[derive(Debug, Clone, Copy)]
pub struct SkipCompact;

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for SkipCompact {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        // suppress all alignment computations, but still correctly report if the row is isolated
        // using a manual computation
        fork_impl::<true, true, _, _, _>(state, align, span, [], Branch::Marker(' '))?;
        Ok((1, minimal.is_empty() || span.len() == 1))
    }
}

/// Either shims a marker, or writes a marker in place of the column while preserving the alignment
/// of the overwritten column.
#[derive(Debug, Clone, Copy)]
pub struct MarkerCompact(pub char);

impl<'a, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>> for MarkerCompact {
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        // suppress all alignment computations, but still correctly report if the row is isolated
        // using a manual computation
        fork_impl::<true, true, _, _, _>(state, align, span, [], Branch::Marker(self.0))?;
        Ok((1, minimal.is_empty() || span.len() == 1))
    }
}

impl<'a, W: io::Write, B: WriteBranch> Shim<&'a mut DiagramWriter<W, B>> for MarkerCompact {
    fn insert(
        self,
        writer: &'a mut DiagramWriter<W, B>,
        gap: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        Marker(self.0).insert(writer, gap)
    }
}

/// A shim which acts as though the column existed in the original database.
pub struct ContinueCompact<'c>(pub &'c mut usize);

impl<'a, 'c, W: io::Write, B: WriteBranch> Apply<&'a mut DiagramWriter<W, B>>
    for ContinueCompact<'c>
{
    type Error = io::Error;

    fn apply<V>(
        self,
        state: &'a mut DiagramWriter<W, B>,
        align: Alignment,
        span: &mut [(V, usize)],
        minimal: MinIndices<'_>,
    ) -> Result<(usize, bool), Self::Error> {
        // supress forks, but still report if a fork was required
        let (shift, _) =
            fork_impl::<false, true, _, _, _>(state, align, span, [], Branch::Continue)?;
        // adjust the column to match the new value
        *self.0 = span.last().unwrap().1;
        Ok((shift, minimal.is_empty() || span.len() == 1))
    }
}

impl<'a, 'c, W: io::Write, B: WriteBranch> Shim<&'a mut DiagramWriter<W, B>>
    for ContinueCompact<'c>
{
    fn insert(
        self,
        state: &'a mut DiagramWriter<W, B>,
        gap: Alignment,
    ) -> Result<(usize, usize, bool), Self::Error> {
        Continue(self.0).insert(state, gap)
    }
}
