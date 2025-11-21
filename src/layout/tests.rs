mod branch_macro;
mod config;
mod fallible;
mod multiline;
mod reverse;
mod single;

use std::fmt::{self, Write};

use crate::{Config, writer::WriteBranch};

use super::*;

#[derive(Clone)]
struct Vtx<T> {
    data: T,
    children: Vec<Vtx<T>>,
}

impl<T> Vtx<T> {
    fn inner(data: T, children: Vec<Vtx<T>>) -> Self {
        Self { data, children }
    }

    fn leaf(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
        }
    }
}

fn assert_diag_impl<R: for<'a> Ramify<&'a Vtx<char>>, B: WriteBranch>(
    root: Vtx<char>,
    expected: &str,
    ramifier: R,
    config: Config<B>,
) {
    println!("\nExpecting tree:\n{expected}");

    let mut writer: Vec<u8> = Vec::new();
    // let mut writer = Writer::new(config, &mut output);
    let mut cols = Generator::init(&root, ramifier, config);
    while cols.write_next_vertex(&mut writer).unwrap() {}

    let received = std::str::from_utf8(&writer).unwrap();

    println!("Got tree:\n{received}");

    assert_eq!(expected, received);
}

fn ex1() -> Vtx<char> {
    let vd = Vtx::leaf('b');
    let vc = Vtx::leaf('c');
    let vb = Vtx::leaf('d');
    let va = Vtx::leaf('a');

    let v9 = Vtx::leaf('9');
    let v8 = Vtx::leaf('8');
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');

    let v4 = Vtx::inner('4', vec![v8]);
    let v3 = Vtx::inner('3', vec![vc, vd, vb]);
    let v2 = Vtx::leaf('2');
    let v1 = Vtx::inner('1', vec![va]);
    Vtx::inner('0', vec![v7, v5, v6, v4, v9, v1, v2, v3])
}

fn ex2() -> Vtx<char> {
    let v8 = Vtx::leaf('8');
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::leaf('3');
    let v2 = Vtx::inner('2', vec![v6]);
    let v1 = Vtx::inner('1', vec![v3]);
    Vtx::inner('0', vec![v7, v1, v2, v5, v4, v8])
}

fn ex3() -> Vtx<char> {
    let v8 = Vtx::leaf('8');
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::leaf('3');
    let v2 = Vtx::inner('2', vec![v6]);
    let v1 = Vtx::inner('1', vec![v3]);
    Vtx::inner('0', vec![v7, v1, v2, v5, v4, v8])
}

fn ex4() -> Vtx<char> {
    let v8 = Vtx::leaf('8');
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::inner('3', vec![v8]);
    let v2 = Vtx::leaf('2');
    let v1 = Vtx::inner('1', vec![v7]);
    Vtx::inner('0', vec![v5, v4, v6, v1, v2, v3])
}

fn ex5() -> Vtx<char> {
    let v8 = Vtx::leaf('8');
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::leaf('3');
    let v2 = Vtx::inner('2', vec![v3, v5]);
    let v1 = Vtx::inner('1', vec![v4, v6]);
    Vtx::inner('0', vec![v2, v1, v7, v8])
}

fn ex6() -> Vtx<char> {
    let v8 = Vtx::leaf('8');
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::inner('3', vec![v8]);
    let v2 = Vtx::inner('2', vec![v7]);
    let v1 = Vtx::leaf('1');
    Vtx::inner('0', vec![v5, v4, v6, v1, v2, v3])
}

fn ex7() -> Vtx<char> {
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
}

fn ex8() -> Vtx<char> {
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
fn ex9() -> Vtx<char> {
    // 0
    // ├┬╮
    // │1├┬╮
    // │││2├┬╮
    // │││││3├╮
    // │││││││4
    // ││││5│││
    // ││6│╭╯││
    // ││ 7│╭╯│
    // ││╭─╯│ 8
    // │││  9 ╭╯
    // │││╭┬╯│
    // ││││a╭╯
    // ││b│╭╯
    // │c╭╯│
    // │││ d
    // ││e
    // │f
    // g
    //
    let vg = Vtx::leaf('g');
    let vf = Vtx::leaf('f');
    let ve = Vtx::leaf('e');
    let vd = Vtx::leaf('d');
    let vc = Vtx::inner('c', vec![vf]);
    let vb = Vtx::leaf('b');
    let va = Vtx::leaf('a');
    let v9 = Vtx::inner('9', vec![ve, va]);
    let v8 = Vtx::inner('8', vec![vd]);
    let v7 = Vtx::leaf('7');
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::inner('4', vec![v8]);
    let v3 = Vtx::inner('3', vec![vb]);
    let v2 = Vtx::inner('2', vec![v7]);
    let v1 = Vtx::inner('1', vec![vc]);
    Vtx::inner('0', vec![vg, v1, v6, v2, v5, v3, v9, v4])
}
