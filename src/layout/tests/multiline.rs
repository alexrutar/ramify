use super::*;

struct Ramifier;

impl<'t> Ramify<&'t Vertex<char>> for Ramifier {
    type Key = char;

    fn children(&mut self, vtx: &'t Vertex<char>) -> impl Iterator<Item = &'t Vertex<char>> {
        vtx.children.iter()
    }

    fn get_key(&self, vtx: &'t Vertex<char>) -> Self::Key {
        vtx.data
    }

    fn marker(&self, vtx: &'t Vertex<char>) -> char {
        vtx.data
    }

    fn annotation<B: fmt::Write>(&self, _: &'t Vertex<char>, _: usize, mut buf: B) -> fmt::Result {
        buf.write_str(">0\n>1\n>2")
    }
}

fn assert_diag(root: Vertex<char>, margin_below: usize, expected: &str) {
    let mut config = Config::<crate::writer::RoundedCorners>::new();
    config.margin_below = margin_below;
    assert_diag_impl(root, expected, Ramifier, config)
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
    assert_diag(
        root,
        0,
        "\
0   >0
╰╮  >1
╭┼╮ >2
│1├╮ >0
││││ >1
││││ >2
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
    assert_diag(
        root.clone(),
        0,
        "\
0   >0
╰╮  >1
╭┼╮ >2
│1├╮ >0
││││ >1
││││ >2
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
    assert_diag(
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
        assert_diag(root, 1, diag);
    }
}
