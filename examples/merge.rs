//! A basic example using reference counting for merges.

use std::{io, rc::Rc};

use ramify::{Generator, Ramify, writer::Style};

/// A directed acyclic graph.
#[derive(Clone)]
struct Vtx {
    data: char,
    // reference counted since the same vertex could appear
    // as the child of multiple vertices
    children: Vec<Rc<Vtx>>,
}

impl Vtx {
    /// A vertex with children.
    fn inner(data: char, children: Vec<Rc<Vtx>>) -> Rc<Self> {
        Rc::new(Self { data, children })
    }

    /// A vertex with no children.
    fn leaf(data: char) -> Rc<Self> {
        Rc::new(Self {
            data,
            children: Vec::new(),
        })
    }
}

/// A ramifier which drains vertices from the graph and writes annotations.
struct AnnotatingRamifier;

impl Ramify<Rc<Vtx>> for AnnotatingRamifier {
    fn ramify(&mut self, vtx: Rc<Vtx>) -> impl IntoIterator<Item = Rc<Vtx>> {
        // unless there are more vertices, this will not clone since the call to
        // this method occurs last, after merging in the children
        Rc::unwrap_or_clone(vtx).children
    }

    fn sort_key(&self, vtx: &Rc<Vtx>) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &Rc<Vtx>) -> char {
        vtx.data
    }

    fn is_identical(&self, vtx: &Rc<Vtx>, other: &Rc<Vtx>) -> bool {
        // we check that the actual vertices are identical instead of just checking the
        // sort key
        Rc::ptr_eq(vtx, other)
    }
}

fn main() -> io::Result<()> {
    let tree = {
        let va = Vtx::leaf('a');
        let v9 = Vtx::leaf('9');
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::inner('6', vec![v7, v8, va]);
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::inner('4', vec![Rc::clone(&v6), Rc::clone(&v9)]);
        let v3 = Vtx::inner('3', vec![v6, Rc::clone(&v4), v5]);
        let v2 = Vtx::inner('2', vec![v3, v9]);
        let v1 = Vtx::inner('1', vec![v2]);
        Vtx::inner('0', vec![v4, v1])
    };

    let mut generator = Generator::new(tree, AnnotatingRamifier);

    // repeatedly write to stdout until the tree is empty
    let mut writer = Style::rounded_corners().io_writer(io::stdout().lock());
    while !generator.is_empty() {
        generator = generator.write(&mut writer)?;
    }

    Ok(())
}
