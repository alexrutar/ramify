//! An example similar to `examples/cascade_merge.rs` but with rate tracking and a much larger
//! tree.
use std::{
    io,
    time::{Duration, Instant},
};

use arrayvec::ArrayVec;
use ramify::{Generator, Ramify, writer::Style};
use rand::{Rng, SeedableRng, seq::IndexedRandom};
use rand_distr::Geometric;
use rand_xoshiro::SplitMix64;

static SEED: u64 = 123456789;
static CHILD_COUNTS_EXPAND: [usize; 5] = [1, 2, 3, 4, 4];
static CHILD_COUNTS_CONTRACT: [usize; 4] = [0, 0, 1, 2];
static WIDTH_TARGET: usize = 100000;

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

    fn annotate(&self, vtx: &u64, buf: &mut String) {
        use std::fmt::Write;
        let _ = write!(buf, "{vtx}");
    }

    fn is_identical(&self, vtx: &u64, other: &u64) -> bool {
        // two vertices are identical if they have the exact same weight
        vtx == other
    }
}

struct Stats {
    bytes_written: usize,
    total_time: Duration,
}

impl Stats {
    fn rate(&self) -> f64 {
        (self.bytes_written as f64) / self.total_time.as_secs_f64() / 1_000_000.0
    }
}

/// A writer which keeps track of the number of bytes written into it, but otherwise discards all
/// writes.
struct RateSink {
    start: Instant,
    bytes: usize,
}

impl RateSink {
    fn start() -> Self {
        Self {
            start: Instant::now(),
            bytes: 0,
        }
    }

    fn sample(&self) -> Stats {
        Stats {
            bytes_written: self.bytes,
            total_time: self.start.elapsed(),
        }
    }
}

impl io::Write for RateSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut sink = RateSink::start();
    let mut writer = Style::rounded_corners().io_writer(&mut sink);
    Generator::new(
        0u64,
        RandomCascade {
            rng: SplitMix64::seed_from_u64(SEED),
            active: 1,
            limit: 10000,
        },
    )
    .write_all(&mut writer)?;
    let stats = sink.sample();
    println!(
        "{} MB written in {} μs",
        (stats.bytes_written as f64) / 1_000_000.0,
        stats.total_time.as_micros()
    );
    println!("{} MB/s", stats.rate());
    Ok(())
}
