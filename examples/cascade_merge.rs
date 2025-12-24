use std::io;

use arrayvec::ArrayVec;
use ramify::{Generator, Ramify, writer::Style};
use rand::{Rng, rngs::ThreadRng, seq::IndexedRandom};
use rand_distr::Geometric;

/// A distribution to increase the number of children; expected 2.2
static CHILD_COUNTS_EXPAND: [usize; 5] = [1, 2, 2, 3, 3];

/// A distribution to decrease the number of children; expected 0.75
static CHILD_COUNTS_CONTRACT: [usize; 4] = [0, 0, 1, 2];

/// A width target which loosely controls how wide the tree will be.
static WIDTH_TARGET: usize = 20;

/// A tree which randomly generates new children.
///
/// Because a `Generator` is streaming, we don't need to store the
/// entire tree in memory.
pub struct RandomCascade {
    rng: ThreadRng,
    active: usize,
    limit: usize,
    /// Set to `true` to show the weight associated with each vertex as an annotation
    show_weight: bool,
}

impl Ramify<u64> for RandomCascade {
    fn ramify(&mut self, vtx: u64) -> impl IntoIterator<Item = u64> {
        // first, decide how many children we generate
        //
        // > if the number is small and we haven't hit the limit, we 'expand'
        //   so that the width grows (in expectation)
        // > otherwise, we 'contract' so that the width shrinks (in expectation)
        //
        // this ensures that the tree never becomes too small or too large, until
        // we hit the limit at which point the tree terminates relatively quickly
        self.limit = self.limit.saturating_sub(1);
        let num_children = if self.active <= WIDTH_TARGET && self.limit > 0 {
            *CHILD_COUNTS_EXPAND.choose(&mut self.rng).unwrap()
        } else {
            *CHILD_COUNTS_CONTRACT.choose(&mut self.rng).unwrap()
        };

        // update the number of active children for next run
        self.active = self.active + num_children - 1;

        let mut array = ArrayVec::<u64, 3>::new();
        for _ in 0..num_children {
            // we generate the new weight by taking the current weight and
            // adding a random positive integer
            let val: u64 = self.rng.sample(Geometric::new(0.25).unwrap());
            array.push(vtx + val + 1);
        }
        array
    }

    fn sort_key(&self, vtx: &u64) -> impl Ord {
        vtx
    }

    fn marker(&self, _: &u64) -> char {
        '◊'
    }

    fn annotate(&self, vtx: &u64, buf: &mut String) {
        if self.show_weight {
            use std::fmt::Write;
            let _ = write!(buf, "{vtx}");
        }
    }

    fn is_identical(&self, vtx: &u64, other: &u64) -> bool {
        // two vertices are identical if they have the exact same weight
        vtx == other
    }
}

fn main() -> std::io::Result<()> {
    let mut generator = Generator::new(
        0u64,
        RandomCascade {
            rng: rand::rng(),
            active: 1,
            limit: 50,          // increase to make the tree larger (on average)
            show_weight: false, // change to `true` to see the vertex weights
        },
    );
    // uncomment these lines to try out some of configuration options
    let mut writer = Style::rounded_corners().io_writer(io::stdout().lock());
    while !generator.is_empty() {
        generator = generator.write_next(&mut writer)?;
    }
    Ok(())
}
