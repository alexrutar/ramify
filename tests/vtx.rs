#![allow(unused)]

pub use std::{collections::HashSet, rc::Rc};

pub use ramify::{Config, Generator, Ramify, State, TryRamify, writer::Style};

#[derive(Clone)]
pub struct Vtx {
    data: char,
    annotation: &'static str,
    children: Vec<Rc<Vtx>>,
}

impl Vtx {
    pub fn leaf(data: char) -> Rc<Self> {
        Self::leaf_annotated(data, "")
    }

    pub fn leaf_annotated(data: char, annotation: &'static str) -> Rc<Self> {
        Rc::new(Self {
            data,
            annotation,
            children: Vec::new(),
        })
    }

    pub fn inner(data: char, children: Vec<Rc<Vtx>>) -> Rc<Self> {
        Self::inner_annotated(data, "", children)
    }

    pub fn inner_annotated(
        data: char,
        annotation: &'static str,
        children: Vec<Rc<Vtx>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            data,
            annotation,
            children,
        })
    }
}

struct Ramifier(&'static str);

impl<'t> Ramify<&'t Rc<Vtx>> for Ramifier {
    fn ramify(&mut self, vtx: &'t Rc<Vtx>) -> impl IntoIterator<Item = &'t Rc<Vtx>> {
        vtx.children.iter()
    }

    fn sort_key(&self, vtx: &&'t Rc<Vtx>) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Rc<Vtx>) -> char {
        vtx.data
    }

    fn annotate(&self, vtx: &&'t Rc<Vtx>, buf: &mut String) {
        buf.push_str(self.0);
        buf.push_str(vtx.annotation);
    }

    fn is_identical(&self, vtx: &&'t Rc<Vtx>, other: &&'t Rc<Vtx>) -> bool {
        Rc::ptr_eq(vtx, other)
    }
}

pub fn assert_diag(
    root: Rc<Vtx>,
    annotation_prefix: &'static str,
    config: Config,
    style: Style,
    expected: &str,
) {
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = style.io_writer(&mut buf);
    let mut cols = Generator::with_config(&root, Ramifier(annotation_prefix), config);
    while !cols.is_empty() {
        cols = cols.write(&mut writer).unwrap()
    }

    let received = std::str::from_utf8(&buf).unwrap();

    assert_eq!(
        expected, received,
        "Expecting tree:\n{expected}\nGot tree:\n{received}"
    );
}

/// A ramifier which fails to iterate the children of a vertex from the provided set.
struct FallibleRamifier(&'static str, HashSet<char>);

impl<'t> TryRamify<Option<&'t Rc<Vtx>>> for FallibleRamifier {
    type Error = ();

    fn try_ramify(
        &mut self,
        vtx: Option<&'t Rc<Vtx>>,
    ) -> Result<impl IntoIterator<Item = Option<&'t Rc<Vtx>>>, Self::Error> {
        match vtx {
            Some(inner) => {
                if self.1.contains(&inner.data) {
                    Err(().into())
                } else {
                    Ok(inner.children.iter().map(Some))
                }
            }
            None => Ok([].iter().map(Some)),
        }
    }

    fn sort_key(&self, vtx: &Option<&'t Rc<Vtx>>) -> impl Ord {
        vtx.map(|v| v.data)
    }

    fn marker(&self, vtx: &Option<&'t Rc<Vtx>>) -> char {
        match vtx {
            Some(inner) => inner.data,
            None => '✕',
        }
    }
}

pub fn assert_diag_fallible<const N: usize>(
    root: Rc<Vtx>,
    failing: [char; N],
    annotation_prefix: &'static str,
    config: Config,
    style: Style,
    expected_err_count: usize,
    expected: &str,
) {
    let mut buf = Vec::new();
    let mut writer = style.io_writer(&mut buf);
    let mut cols = Generator::with_config(
        Some(&root),
        FallibleRamifier(annotation_prefix, failing.into_iter().collect()),
        config,
    );

    let mut n = 0;
    while !cols.is_empty() {
        cols = match cols.try_write(&mut writer).unwrap() {
            State::Ok(generator) => generator,
            State::Suspended(suspended, ()) => {
                n += 1;
                suspended.resume(&mut writer, [None]).unwrap()
            }
        };
    }

    let received = String::from_utf8(buf).unwrap();
    assert_eq!(
        expected, received,
        "Expecting tree:\n{expected}\nGot tree:\n{received}"
    );
    assert_eq!(expected_err_count, n);
}

pub fn ex1() -> Rc<Vtx> {
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

pub fn ex2() -> Rc<Vtx> {
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

pub fn ex3() -> Rc<Vtx> {
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

pub fn ex4() -> Rc<Vtx> {
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

pub fn ex5() -> Rc<Vtx> {
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

pub fn ex6() -> Rc<Vtx> {
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

pub fn ex7() -> Rc<Vtx> {
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

pub fn ex8() -> Rc<Vtx> {
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

pub fn ex9() -> Rc<Vtx> {
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
