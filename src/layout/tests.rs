mod branch_macro;
mod config;
mod multiline;
mod single;

use std::fmt::{self, Write};

use crate::{Config, writer::WriteBranch};

use super::*;

#[derive(Clone)]
struct Vertex<T> {
    data: T,
    children: Vec<Vertex<T>>,
}

impl<T> Vertex<T> {
    fn inner(data: T, children: Vec<Vertex<T>>) -> Self {
        Self { data, children }
    }

    fn leaf(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
        }
    }
}

fn assert_diag_impl<R: for<'a> Ramify<&'a Vertex<char>>, B: WriteBranch>(
    root: Vertex<char>,
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

fn ex1() -> Vertex<char> {
    let vd = Vertex::leaf('b');
    let vc = Vertex::leaf('c');
    let vb = Vertex::leaf('d');
    let va = Vertex::leaf('a');

    let v9 = Vertex::leaf('9');
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');

    let v4 = Vertex::inner('4', vec![v8]);
    let v3 = Vertex::inner('3', vec![vc, vd, vb]);
    let v2 = Vertex::leaf('2');
    let v1 = Vertex::inner('1', vec![va]);
    Vertex::inner('0', vec![v7, v5, v6, v4, v9, v1, v2, v3])
}

fn ex2() -> Vertex<char> {
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');
    let v4 = Vertex::leaf('4');
    let v3 = Vertex::leaf('3');
    let v2 = Vertex::inner('2', vec![v6]);
    let v1 = Vertex::inner('1', vec![v3]);
    Vertex::inner('0', vec![v7, v1, v2, v5, v4, v8])
}

fn ex3() -> Vertex<char> {
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');
    let v4 = Vertex::leaf('4');
    let v3 = Vertex::leaf('3');
    let v2 = Vertex::inner('2', vec![v6]);
    let v1 = Vertex::inner('1', vec![v3]);
    Vertex::inner('0', vec![v7, v1, v2, v5, v4, v8])
}

fn ex4() -> Vertex<char> {
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');
    let v4 = Vertex::leaf('4');
    let v3 = Vertex::inner('3', vec![v8]);
    let v2 = Vertex::leaf('2');
    let v1 = Vertex::inner('1', vec![v7]);
    Vertex::inner('0', vec![v5, v4, v6, v1, v2, v3])
}

fn ex5() -> Vertex<char> {
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');
    let v4 = Vertex::leaf('4');
    let v3 = Vertex::leaf('3');
    let v2 = Vertex::inner('2', vec![v3, v5]);
    let v1 = Vertex::inner('1', vec![v4, v6]);
    Vertex::inner('0', vec![v2, v1, v7, v8])
}

fn ex6() -> Vertex<char> {
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');
    let v4 = Vertex::leaf('4');
    let v3 = Vertex::inner('3', vec![v8]);
    let v2 = Vertex::inner('2', vec![v7]);
    let v1 = Vertex::leaf('1');
    Vertex::inner('0', vec![v5, v4, v6, v1, v2, v3])
}

fn ex7() -> Vertex<char> {
    let v8 = Vertex::leaf('8');
    let v7 = Vertex::leaf('7');
    let v6 = Vertex::leaf('6');
    let v5 = Vertex::leaf('5');
    let v4 = Vertex::leaf('4');
    let v3 = Vertex::inner('3', vec![v8]);
    let v2 = Vertex::leaf('2');
    let v1 = Vertex::inner('1', vec![v7]);
    Vertex::inner('0', vec![v5, v4, v6, v1, v2, v3])
}
