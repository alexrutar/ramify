use criterion::{Criterion, criterion_group, criterion_main};
use ramify::{Config, Ramify, writer::Style};
use std::hint::black_box;

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

struct Ramifier;

impl<'t> Ramify<&'t Vtx> for Ramifier {
    fn ramify(&mut self, vtx: &'t Vtx) -> impl IntoIterator<Item = &'t Vtx> {
        vtx.children.iter()
    }

    fn sort_key(&self, vtx: &&'t Vtx) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Vtx) -> char {
        vtx.data
    }
}

fn tree() -> Vtx {
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
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut writer = Style::rounded_corners().io_writer(std::io::empty());
    let root = tree();
    c.bench_function("recursive", |b| {
        b.iter(|| {
            Config::new()
                .row_padding(1)
                .generator(black_box(&root), Ramifier)
                .write_all(black_box(&mut writer))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
