use std::fmt::{self, Write};

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

fn assert_diag_impl<R: for<'a> Ramify<&'a Vertex<char>>>(
    root: Vertex<char>,
    expected: &str,
    ramifier: R,
    padding: usize,
) {
    println!("\nExpecting tree:\n{expected}");

    let mut output: Vec<u8> = Vec::new();
    let mut writer = Writer::with_default_config(&mut output);
    writer.config.margin_below = padding;
    let mut cols = Generator::init(&root, ramifier);
    while cols.write_diagram_row(&mut writer).unwrap() {}

    let received = std::str::from_utf8(&output).unwrap();

    println!("Got tree:\n{received}");

    assert_eq!(expected, received);
}

struct CharTreeRamifier;

impl<'t> Ramify<&'t Vertex<char>> for CharTreeRamifier {
    type Key = char;

    fn children(&self, vtx: &'t Vertex<char>) -> impl Iterator<Item = &'t Vertex<char>> {
        vtx.children.iter()
    }

    fn get_key(&self, vtx: &'t Vertex<char>) -> Self::Key {
        vtx.data
    }

    fn marker(&self, vtx: &'t Vertex<char>) -> char {
        vtx.data
    }

    fn annotation<B: Write>(&self, _: &'t Vertex<char>, _: usize, mut buf: B) {
        let _ = buf.write_char('#');
    }
}

fn assert_diag(root: Vertex<char>, expected: &str) {
    assert_diag_impl(root, expected, CharTreeRamifier, 0)
}

#[test]
fn lookahead() {
    let root = {
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::leaf('3');
        let v2 = Vertex::inner('2', vec![v4, v3, v5]);
        let v1 = Vertex::inner('1', vec![v6]);
        Vertex::inner('0', vec![v2, v1, v7])
    };

    assert_diag(
        root,
        "\
0   #
├┬╮
│1│ #
2│╰─╮ #
│╰─╮│
├┬╮││
│3│││ #
4╭╯││ #
 5╭╯│ #
  6╭╯ #
   7 #
",
    );
}

#[test]
fn small() {
    let expected_diags = [
        "\
0  #
├╮
1│ #
╭┤
2│ #
 3 #
",
        "\
0  #
├╮
1│ #
╭┤
│2 #
3 #
",
        "\
0   #
├┬╮
│1│ #
2╭╯ #
 3 #
",
        "\
0  #
├╮
│1 #
├╮
2│ #
 3 #
",
        "\
0   #
├┬╮
│1│ #
│ 2 #
3 #
",
        "\
0  #
├╮
│1 #
├╮
│2 #
3 #
",
    ];

    for ((c1, c2, c3), diag) in [
        ('1', '2', '3'),
        ('1', '3', '2'),
        ('2', '1', '3'),
        ('2', '3', '1'),
        ('3', '1', '2'),
        ('3', '2', '1'),
    ]
    .into_iter()
    .zip(expected_diags)
    {
        let root = {
            let v1 = Vertex::leaf(c1);
            let v2 = Vertex::leaf(c2);
            let v3 = Vertex::leaf(c3);
            Vertex::inner('0', vec![v1, v2, v3])
        };
        assert_diag(root, diag);
    }
}

#[test]
fn long_skip() {
    let root = {
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::inner('5', vec![v7]);
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::leaf('3');
        let v2 = Vertex::inner('2', vec![v3]);
        let v1 = Vertex::inner('1', vec![v4, v5]);
        Vertex::inner('0', vec![v1, v2, v6])
    };
    assert_diag(
        root,
        "\
0  #
├╮
1├╮ #
│2│ #
│3│ #
├╮│
4││ #
 5│ #
╭╯6 #
7 #
",
    );
}

#[test]
fn long_inner_path() {
    let root = {
        let v8 = Vertex::leaf('8');
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::inner('3', vec![v8]);
        let v2 = Vertex::leaf('2');
        let v1 = Vertex::inner('1', vec![v7]);
        Vertex::inner('0', vec![v5, v4, v6, v1, v2, v3])
    };
    assert_diag(
        root,
        "\
0   #
├┬╮
│1├╮ #
││2│ #
│╰╮3  #
│ │╰╮
│ ╰╮│
├┬╮││
│4│││ #
5╭╯││ #
 6╭╯│ #
  7╭╯ #
   8 #
",
    );
}

#[test]
fn width_jitter() {
    let root = {
        let v8 = Vertex::leaf('8');
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::inner('3', vec![v8]);
        let v2 = Vertex::inner('2', vec![v7]);
        let v1 = Vertex::leaf('1');
        Vertex::inner('0', vec![v5, v4, v6, v1, v2, v3])
    };
    assert_diag(
        root,
        "\
0   #
├┬╮
│1│ #
│╭┤
│2│ #
││3   #
││╰─╮
│╰─╮│
├┬╮││
│4│││ #
5╭╯││ #
 6╭╯│ #
  7╭╯ #
   8 #
",
    );
}

#[test]
fn no_lookahead() {
    let root = {
        let v8 = Vertex::leaf('8');
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::leaf('3');
        let v2 = Vertex::inner('2', vec![v3, v5]);
        let v1 = Vertex::inner('1', vec![v4, v6]);
        Vertex::inner('0', vec![v2, v1, v7, v8])
    };
    assert_diag(
        root,
        "\
0   #
├┬╮
│1│ #
2│╰╮ #
│╰╮│
├╮││
3│││ #
╭╯││
│╭┤│
│4││ #
5╭╯│ #
 6╭┤ #
  7│ #
   8 #
",
    );
}

#[test]
fn complex_width_computations() {
    let root = {
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
    };
    assert_diag(
        root,
        "\
0   #
├┬╮
│1├╮ #
││2│ #
│╰╮3  #
│ │╰╮
│ ╰╮│
├┬╮││
│4││╰─╮ #
│││╰─╮│
││╰─╮││
│╰─╮│││
├┬╮││││
│5│││││ #
│ 6││││ #
7╭─╯│││ #
 8╭─╯││ #
  9╭─╯│ #
   a╭┬┤ #
╭───╯b│ #
c╭────╯ #
 d #
",
    );
}

#[test]
fn annotation_whitespace_management() {
    let root = {
        let v8 = Vertex::leaf('8');
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::leaf('3');
        let v2 = Vertex::inner('2', vec![v6]);
        let v1 = Vertex::inner('1', vec![v3]);
        Vertex::inner('0', vec![v7, v1, v2, v5, v4, v8])
    };
    assert_diag(
        root,
        "\
0   #
├┬╮
│1├╮ #
││2│ #
│3││  #
│╭╯│ 
││╭┼╮
│││4│ #
││5╭╯ #
│6╭╯ #
7╭╯ #
 8 #
",
    );
}

struct MultiAnnotationTreeRamifier;

impl<'t> Ramify<&'t Vertex<char>> for MultiAnnotationTreeRamifier {
    type Key = char;

    fn children(&self, vtx: &'t Vertex<char>) -> impl Iterator<Item = &'t Vertex<char>> {
        vtx.children.iter()
    }

    fn get_key(&self, vtx: &'t Vertex<char>) -> Self::Key {
        vtx.data
    }

    fn marker(&self, vtx: &'t Vertex<char>) -> char {
        vtx.data
    }

    fn annotation<B: fmt::Write>(&self, _: &'t Vertex<char>, _: usize, mut buf: B) {
        let _ = buf.write_str(">0\n>1\n>2");
    }
}

fn assert_diag_multi(root: Vertex<char>, padding: usize, expected: &str) {
    assert_diag_impl(root, expected, MultiAnnotationTreeRamifier, padding)
}

#[test]
fn multiline_annotations() {
    let root = {
        let v8 = Vertex::leaf('8');
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::leaf('3');
        let v2 = Vertex::inner('2', vec![v6]);
        let v1 = Vertex::inner('1', vec![v3]);
        Vertex::inner('0', vec![v7, v1, v2, v5, v4, v8])
    };
    assert_diag_multi(
        root,
        0,
        "\
0   >0
╰╮  >1
╭┼╮ >2
│1│  >0
│││  >1
││├╮ >2
││2│ >0
││││ >1
││││ >2
│3││  >0
│╭╯│  >1
││╭┼╮ >2
│││4│ >0
│││╭╯ >1
││││  >2
││5│ >0
││╭╯ >1
│││  >2
│6│ >0
│╭╯ >1
││  >2
7│ >0
╭╯ >1
│  >2
8 >0
  >1
  >2
",
    );
}

#[test]
fn inner_path_multiline_padded() {
    let root = {
        let v8 = Vertex::leaf('8');
        let v7 = Vertex::leaf('7');
        let v6 = Vertex::leaf('6');
        let v5 = Vertex::leaf('5');
        let v4 = Vertex::leaf('4');
        let v3 = Vertex::inner('3', vec![v8]);
        let v2 = Vertex::leaf('2');
        let v1 = Vertex::inner('1', vec![v7]);
        Vertex::inner('0', vec![v5, v4, v6, v1, v2, v3])
    };
    assert_diag_multi(
        root.clone(),
        0,
        "\
0   >0
╰╮  >1
╭┼╮ >2
│1│  >0
│││  >1
││├╮ >2
││2│ >0
││╭╯ >1
│││  >2
││3   >0
││╰─╮ >1
│╰─╮│ >2
├┬╮││
│4│││ >0
│╭╯││ >1
││╭╯│ >2
5││╭╯ >0
╭╯││  >1
│╭╯│  >2
6│╭╯ >0
╭╯│  >1
│╭╯  >2
7│ >0
╭╯ >1
│  >2
8 >0
  >1
  >2
",
    );
    assert_diag_multi(
        root,
        1,
        "\
0   >0
╰╮  >1
╭┼╮ >2
│││
│1│  >0
│││  >1
││├╮ >2
││││
││2│ >0
││╭╯ >1
│││  >2
│││
││3   >0
││╰─╮ >1
│╰─╮│ >2
├┬╮││
│4│││ >0
│╭╯││ >1
││╭╯│ >2
│││╭╯
5│││ >0
╭╯││ >1
│╭╯│ >2
││╭╯
6││ >0
╭╯│ >1
│╭╯ >2
││
7│ >0
╭╯ >1
│  >2
│
8 >0
  >1
  >2
",
    );
}

#[test]
fn small_multi() {
    let expected_diags = [
        "\
0  >0
│  >1
├╮ >2
││
1│ >0
╭╯ >1
├╮ >2
││
2│ >0
╭╯ >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0  >0
│  >1
├╮ >2
││
1│ >0
 │ >1
╭┤ >2
││
│2 >0
│  >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0   >0
╰╮  >1
╭┼╮ >2
│││
│1│ >0
│╭╯ >1
││  >2
││
2│ >0
╭╯ >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0  >0
╰╮ >1
╭┤ >2
││
│1 >0
│  >1
├╮ >2
││
2│ >0
╭╯ >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0   >0
╰╮  >1
╭┼╮ >2
│││
│1│ >0
│╭╯ >1
││  >2
││
│2 >0
│  >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0  >0
╰╮ >1
╭┤ >2
││
│1 >0
╰╮ >1
╭┤ >2
││
│2 >0
│  >1
│  >2
│
3 >0
  >1
  >2
",
    ];

    for ((c1, c2, c3), diag) in [
        ('1', '2', '3'),
        ('1', '3', '2'),
        ('2', '1', '3'),
        ('2', '3', '1'),
        ('3', '1', '2'),
        ('3', '2', '1'),
    ]
    .into_iter()
    .zip(expected_diags)
    {
        let root = {
            let v1 = Vertex::leaf(c1);
            let v2 = Vertex::leaf(c2);
            let v3 = Vertex::leaf(c3);
            Vertex::inner('0', vec![v1, v2, v3])
        };
        assert_diag_multi(root, 1, diag);
    }
}
