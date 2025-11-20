//! A basic example using a recursive in-memory tree, but with rendering which can 'fail'.

use std::io;

use ramify::{Config, Generator, TryRamify, WriteVertexError};
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

impl<'t> TryRamify<Option<&'t Vtx>> for FallibleRamifier {
    /// Here, the `None` variant implies that rendering has failed.
    type Key = Option<char>;

    fn try_children(
        &mut self,
        vtx: Option<&'t Vtx>,
    ) -> Result<impl IntoIterator<Item = Option<&'t Vtx>>, Option<&'t Vtx>> {
        match vtx {
            Some(v) => {
                // with probability 0.3, rendering "fails", unless we are at vertex 0
                let d = Bernoulli::new(0.3).unwrap();
                if v.data != '0' && d.sample(&mut rand::rng()) {
                    Err(None)
                } else {
                    Ok(v.children.iter().map(Some))
                }
            }
            // a failed vertex has no children
            None => Ok([].iter().map(Some)),
        }
    }

    fn get_key(&self, vtx: &Option<&'t Vtx>) -> Self::Key {
        vtx.map(|v| v.data)
    }

    fn marker(&self, vtx: &Option<&'t Vtx>) -> char {
        match vtx {
            Some(v) => v.data,
            // a visual representation of 'failed' rendering
            None => '✕',
        }
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

    let config = Config::with_rounded_corners();

    // initially the tree is in a 'good' state
    let mut diag = Generator::init(Some(&tree), FallibleRamifier, config);

    // repeatedly write to stdout until the tree is empty
    let mut writer = io::stdout();
    loop {
        match diag.try_write_next_vertex(&mut writer) {
            // rendering succeed
            Ok(true) => {}
            // no more elements in the tree
            Ok(false) => break,
            // failed to write to stdout; bail
            Err(WriteVertexError::IO(e)) => return Err(e),
            // if rendering failed, we just retry it
            Err(WriteVertexError::TryChildrenFailed) => {}
        }
    }

    Ok(())
}
