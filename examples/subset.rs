//! A basic example of rendering which may (randomly) fail in which case iteration is aborted
//! immediately.

use std::io;

use ramify::{Config, Generator, Ramify};
use rand::distr::{Bernoulli, Distribution};

/// A basic recursive tree implementation.
struct Vtx {
    data: char,
    children: Vec<Vtx>,
}

impl Vtx {
    /// A vertex with children.
    fn inner(data: char, children: Vec<Vtx>) -> Self {
        Self { data, children }
    }

    /// A vertex with no children.
    fn leaf(data: char) -> Self {
        Self {
            data,
            children: Vec::new(),
        }
    }
}

/// A ramifier which randomly fails to iterate over children of a given vertex
struct Ramifier;

impl<'t> Ramify<&'t Vtx> for Ramifier {
    fn ramify(&mut self, vtx: &'t Vtx) -> impl IntoIterator<Item = &'t Vtx> {
        // with probability 0.3, rendering "fails", except at the first level
        let d = Bernoulli::new(0.3).unwrap();
        if vtx.data != '0' && d.sample(&mut rand::rng()) {
            [].iter()
        } else {
            vtx.children.iter()
        }
    }

    fn sort_key(&self, vtx: &&'t Vtx) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Vtx) -> char {
        vtx.data
    }
}

fn main() -> io::Result<()> {
    // a big tree to make it easier to see what is happening
    let tree = {
        let vg = Vtx::leaf('g');
        let vf = Vtx::leaf('f');
        let ve = Vtx::leaf('e');
        let vd = Vtx::leaf('d');
        let vc = Vtx::inner('c', vec![vg]);
        let vb = Vtx::inner('b', vec![vf, vd]);
        let va = Vtx::leaf('a');
        let v9 = Vtx::leaf('9');
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::inner('7', vec![vb]);
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::inner('3', vec![v7]);
        let v2 = Vtx::inner('2', vec![v6]);
        let v1 = Vtx::inner('1', vec![v9, v5]);
        Vtx::inner('0', vec![va, vc, v1, v4, v2, v8, v3, ve])
    };

    let config = Config::new();

    // initially the tree is in a 'good' state
    let mut diag = Generator::with_config(&tree, Ramifier, config);

    // repeatedly write to stdout until the tree is empty
    let mut writer = ramify::writer::Style::rounded_corners().io_writer(io::stdout().lock());
    while !diag.is_empty() {
        diag = diag.write_next(&mut writer)?;
    }

    Ok(())
}
