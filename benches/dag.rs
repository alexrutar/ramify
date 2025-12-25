use std::hint::black_box;

use arrayvec::ArrayVec;
use criterion::{Criterion, criterion_group, criterion_main};
use ramify::{Config, Generator, Ramify, writer::Style};
use rand::{Rng, SeedableRng, seq::IndexedRandom};
use rand_distr::Geometric;
use rand_xoshiro::SplitMix64;

/// The seed for deterministic output.
static SEED: u64 = 123456789;

/// A distribution to increase the number of children; expected 2.8
static CHILD_COUNTS_EXPAND: [usize; 5] = [1, 2, 3, 4, 4];

/// A distribution to decrease the number of children; expected 0.75
static CHILD_COUNTS_CONTRACT: [usize; 4] = [0, 0, 1, 2];

/// A width target which loosely controls how wide the tree will be.
static WIDTH_TARGET: usize = 100000;

/// A tree which randomly generates new children.
///
/// Because a `Generator` is streaming, we don't need to store the
/// entire tree in memory.
pub struct RandomCascade {
    rng: SplitMix64,
    active: usize,
    limit: usize,
}

impl Ramify<u64> for RandomCascade {
    fn ramify(&mut self, vtx: u64) -> impl IntoIterator<Item = u64> {
        self.limit = self.limit.saturating_sub(1);
        let num_children = if self.active <= WIDTH_TARGET && self.limit > 0 {
            *CHILD_COUNTS_EXPAND.choose(&mut self.rng).unwrap()
        } else {
            *CHILD_COUNTS_CONTRACT.choose(&mut self.rng).unwrap()
        };

        self.active = self.active + num_children - 1;

        let mut array = ArrayVec::<u64, 4>::new();
        for _ in 0..num_children {
            let val: u64 = self.rng.sample(Geometric::new(0.1).unwrap());
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

    fn annotate(&self, _: &u64, buf: &mut String) {
        buf.push_str("0\n1\n2");
    }

    fn is_identical(&self, vtx: &u64, other: &u64) -> bool {
        // two vertices are identical if they have the exact same weight
        vtx == other
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut writer = Style::rounded_corners().io_writer(std::io::empty());
    c.bench_function("dag default", |b| {
        b.iter(|| {
            Generator::new(
                0u64,
                RandomCascade {
                    rng: black_box(SplitMix64::seed_from_u64(SEED)),
                    active: 1,
                    limit: 10000,
                },
            )
            .write_all(black_box(&mut writer))
        })
    });

    c.bench_function("dag inverted", |b| {
        b.iter(|| {
            Config::new()
                .inverted_annotations(true)
                .generator(
                    0u64,
                    RandomCascade {
                        rng: black_box(SplitMix64::seed_from_u64(SEED)),
                        active: 1,
                        limit: 10000,
                    },
                )
                .write_all(black_box(&mut writer))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
