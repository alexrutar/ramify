use crate::writer::{RoundedCorners, RoundedCornersWide, SharpCorners, SharpCornersWide};

use super::*;

fn assert_diag<B: WriteBranch>(root: Vertex<char>, config: Config<B>, expected: &str) {
    struct Ramifier;

    impl<'t> Ramify<&'t Vertex<char>> for Ramifier {
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
    }

    assert_diag_impl(root, expected, Ramifier, config)
}

fn assert_diag_annot<B: WriteBranch>(root: Vertex<char>, config: Config<B>, expected: &str) {
    struct AnnotatingRamifier;

    impl<'t> Ramify<&'t Vertex<char>> for AnnotatingRamifier {
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

        fn annotation<B: fmt::Write>(
            &self,
            _: &'t Vertex<char>,
            tree_width: usize,
            mut buf: B,
        ) -> fmt::Result {
            write!(buf, "{tree_width}")
        }
    }

    assert_diag_impl(root, expected, AnnotatingRamifier, config)
}

#[test]
fn annotation_style() {
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

    let config = Config::<RoundedCorners>::new();
    assert_diag(
        root.clone(),
        config,
        "\
0
├┬╮
│1├╮
││2│
│3││
│╭╯│
││╭┼╮
│││4│
││5╭╯
│6╭╯
7╭╯
 8
",
    );

    let config = Config::<SharpCorners>::new();
    assert_diag(
        root.clone(),
        config,
        "\
0
├┬┐
│1├┐
││2│
│3││
│┌┘│
││┌┼┐
│││4│
││5┌┘
│6┌┘
7┌┘
 8
",
    );

    let config = Config::<RoundedCornersWide>::new();
    assert_diag(
        root.clone(),
        config,
        "\
0
├─┬─╮
│ 1 ├─╮
│ │ 2 │
│ 3 │ │
│ ╭─╯ │
│ │ ╭─┼─╮
│ │ │ 4 │
│ │ 5 ╭─╯
│ 6 ╭─╯
7 ╭─╯
  8
",
    );

    let config = Config::<SharpCornersWide>::new();
    assert_diag_annot(
        root.clone(),
        config,
        "\
0     6
├─┬─┐
│ 1 ├─┐ 8
│ │ 2 │ 8
│ 3 │ │   10
│ ┌─┘ │
│ │ ┌─┼─┐
│ │ │ 4 │ 10
│ │ 5 ┌─┘ 10
│ 6 ┌─┘ 8
7 ┌─┘ 6
  8 4
",
    );
}

#[test]
fn annotation_reported_line_width() {
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

    assert_diag_annot(
        root.clone(),
        Config::<RoundedCorners>::new(),
        "\
0   4
├┬╮
│1├╮ 5
││2│ 5
│╰╮3  6
│ │╰╮
│ ╰╮│
├┬╮││
│4││╰─╮ 8
│││╰─╮│
││╰─╮││
│╰─╮│││
├┬╮││││
│5│││││ 8
│ 6││││ 8
7╭─╯│││ 8
 8╭─╯││ 8
  9╭─╯│ 8
   a╭┬┤ 8
╭───╯b│ 8
c╭────╯ 8
 d 3
",
    );

    let config = Config::<RoundedCornersWide>::new();
    assert_diag_annot(
        root.clone(),
        config,
        "\
0     6
├─┬─╮
│ 1 ├─╮ 8
│ │ 2 │ 8
│ ╰─╮ 3   10
│   │ ╰─╮
│   ╰─╮ │
├─┬─╮ │ │
│ 4 │ │ ╰───╮ 14
│ │ │ ╰───╮ │
│ │ ╰───╮ │ │
│ ╰───╮ │ │ │
├─┬─╮ │ │ │ │
│ 5 │ │ │ │ │ 14
│   6 │ │ │ │ 14
7 ╭───╯ │ │ │ 14
  8 ╭───╯ │ │ 14
    9 ╭───╯ │ 14
      a ╭─┬─┤ 14
╭───────╯ b │ 14
c ╭─────────╯ 14
  d 4
",
    );
}
