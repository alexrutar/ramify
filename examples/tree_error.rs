//! An example with errors encapsulated inside the tree itself.

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

struct Children<'a> {
    inner: std::slice::Iter<'a, Vtx>,
}

impl<'a> Iterator for Children<'a> {
    type Item = Result<&'a Vtx, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.inner.next()?;

        let d = Bernoulli::new(0.2).unwrap();
        Some(if d.sample(&mut rand::rng()) {
            Err(())
        } else {
            Ok(v)
        })
    }
}

/// A ramifier which randomly fails to iterate over children of a given vertex and displays a
/// custom failure marker.
struct Ramifier;

impl<'t> Ramify<Result<&'t Vtx, ()>> for Ramifier {
    fn ramify(
        &mut self,
        maybe_vtx: Result<&'t Vtx, ()>,
    ) -> impl IntoIterator<Item = Result<&'t Vtx, ()>> {
        match maybe_vtx {
            Ok(vtx) => Children {
                inner: vtx.children.iter(),
            },
            Err(()) => Children { inner: [].iter() },
        }
    }

    fn sort_key(&self, vtx: &Result<&'t Vtx, ()>) -> impl Ord {
        // use option to sort failures first
        vtx.map(|v| v.data).ok()
    }

    fn marker(&self, vtx: &Result<&'t Vtx, ()>) -> char {
        vtx.map(|v| v.data).unwrap_or('✕')
    }
}

fn main() -> io::Result<()> {
    // a big tree to make it easier to see what is happening
    let tree = {
        let vh = Vtx::leaf('h');
        let vg = Vtx::leaf('g');
        let vf = Vtx::leaf('f');
        let ve = Vtx::leaf('e');
        let vd = Vtx::inner('d', vec![vh]);
        let vc = Vtx::inner('c', vec![vg]);
        let vb = Vtx::inner('b', vec![vf]);
        let va = Vtx::inner('a', vec![ve, vb]);
        let v9 = Vtx::inner('9', vec![vd]);
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::inner('7', vec![vc]);
        let v6 = Vtx::inner('6', vec![va, v7]);
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::inner('4', vec![v6]);
        let v3 = Vtx::inner('3', vec![v5]);
        let v2 = Vtx::inner('2', vec![v3, v9]);
        let v1 = Vtx::inner('1', vec![v4, v2]);
        Vtx::inner('0', vec![v8, v1])
    };

    let config = Config::new();

    // initially the tree is in a 'good' state
    let mut diag = Generator::with_config(Ok(&tree), Ramifier, config);

    // repeatedly write to stdout until the tree is empty
    let mut writer = ramify::writer::Style::rounded_corners().io_writer(io::stdout().lock());
    while !diag.is_empty() {
        diag = diag.write(&mut writer)?;
    }

    Ok(())
}
