//! A basic example of rendering which may (randomly) fail in which case iteration is aborted
//! immediately.
//!
//! Compare this to the `subset` example, which does not abort on error and instead silently
//! omits the children.

use std::io;

use ramify::{Generator, State, TryRamify, writer::Style};
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

/// A ramifier which randomly fails to compute the children for a given vertex
struct FallibleRamifier;

impl<'t> TryRamify<&'t Vtx> for FallibleRamifier {
    /// In practice we would include more information here.
    type Error = char;

    fn try_ramify(
        &mut self,
        vtx: &'t Vtx,
    ) -> Result<impl IntoIterator<Item = &'t Vtx>, Self::Error> {
        // with probability 0.15, rendering "fails"
        let d = Bernoulli::new(0.15).unwrap();
        if d.sample(&mut rand::rng()) {
            Err(vtx.data)
        } else {
            Ok(vtx.children.iter())
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

    // initially the tree is in a 'good' state
    let mut diag = Generator::new(&tree, FallibleRamifier);

    let mut writer = Style::rounded_corners().io_writer(io::stdout().lock());
    while !diag.is_empty() {
        diag = match diag.try_write(&mut writer)? {
            State::Ok(generator) => generator,
            State::Suspended(suspended, ch) => {
                // write the last vertex before the error
                suspended.resume(&mut writer, std::iter::empty())?;
                println!("Failed to determine children of vertex '{ch}': aborting!");
                break;
            }
        };
    }

    Ok(())
}
