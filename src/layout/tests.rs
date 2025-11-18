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
