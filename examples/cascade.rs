use std::io;

use arrayvec::ArrayVec;
use ramify::{Generator, Ramify};
use rand::{Rng, rngs::ThreadRng, seq::IndexedRandom};
use rand_distr::Exp1;

/// A wrapper around an `f64` using `total_cmp` for ordering
pub struct Key(f64);

/// Ord boilerplate
mod key {
    use super::Key;
    use std::cmp::Ordering;

    impl PartialEq for Key {
        fn eq(&self, other: &Self) -> bool {
            self.0.total_cmp(&other.0) == Ordering::Equal
        }
    }

    impl Eq for Key {}

    impl PartialOrd for Key {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.0.total_cmp(&other.0))
        }
    }

    impl Ord for Key {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.total_cmp(&other.0)
        }
    }
}

/// A distribution to increase the number of children; expected 1.4
static CHILD_COUNTS_EXPAND: [usize; 5] = [0, 1, 1, 2, 3];

/// A distribution to decrease the number of children; expected 0.75
static CHILD_COUNTS_CONTRACT: [usize; 4] = [0, 0, 1, 2];

/// A tree which randomly generates new children.
///
/// Because a `Generator` is streaming, we don't need to store the
/// entire tree in memory.
pub struct RandomCascade {
    rng: ThreadRng,
    active: usize,
    limit: usize,
}

impl Ramify<f64> for RandomCascade {
    type Key = Key;

    fn children(&mut self, vtx: f64) -> impl IntoIterator<Item = f64> {
        // first, decide how many children we generate
        //
        // > if the number is small and we haven't hit the limit, we 'expand'
        //   so that the width grows (in expectation)
        // > otherwise, we 'contract' so that the width shrinks (in expectation)
        //
        // this ensures that the tree never becomes too small or too large, until
        // we hit the limit at which point the tree terminates relatively quickly
        self.limit = self.limit.saturating_sub(1);
        let num_children = if self.active <= 5 && self.limit > 0 {
            *CHILD_COUNTS_EXPAND.choose(&mut self.rng).unwrap()
        } else {
            *CHILD_COUNTS_CONTRACT.choose(&mut self.rng).unwrap()
        };

        // update the number of active children for next run
        self.active = self.active + num_children - 1;

        let mut array = ArrayVec::<f64, 3>::new();
        for _ in 0..num_children {
            // we generate the new weights by taking the current weight and
            // add a random positive number sampled from the exponential
            // distribution
            //
            // you can try a different distribution here; e.g.
            //
            // rand_distr::Weibull
            // rand_distr::Frechet
            let val: f64 = self.rng.sample(Exp1);
            array.push(vtx + val);
        }
        array
    }

    fn get_key(&self, vtx: &f64) -> Self::Key {
        Key(*vtx)
    }

    fn marker(&self, _: &f64) -> char {
        '◊'
    }
}

fn main() -> std::io::Result<()> {
    let mut generator = Generator::with_rounded_corners(
        0f64,
        RandomCascade {
            rng: rand::rng(),
            active: 1,
            limit: 50,
        },
    );
    // modify these lines to try out some of the configuration options
    // generator.config_mut().row_padding = 1;
    // generator.config_mut().width_slack = true;
    while generator.write_next_vertex(io::stdout().lock())? {}
    Ok(())
}
