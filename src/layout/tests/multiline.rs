use super::*;

fn assert_diag(root: Vtx<char>, margin_below: usize, expected: &str) {
    struct Ramifier;

    impl<'t> Ramify<&'t Vtx<char>> for Ramifier {
        type Key = char;

        fn children(&mut self, vtx: &'t Vtx<char>) -> impl IntoIterator<Item = &'t Vtx<char>> {
            vtx.children.iter()
        }

        fn get_key(&self, vtx: &&'t Vtx<char>) -> Self::Key {
            vtx.data
        }

        fn marker(&self, vtx: &&'t Vtx<char>) -> char {
            vtx.data
        }

        fn annotation<B: fmt::Write>(&self, _: &&'t Vtx<char>, mut buf: B) -> fmt::Result {
            buf.write_str(">0\n>1\n>2")
        }
    }

    let mut config = Config::<crate::writer::RoundedCorners>::new();
    config.row_padding = margin_below;
    assert_diag_impl(root, expected, Ramifier, config)
}

#[test]
fn multiline_annotations() {
    assert_diag(
        ex3(),
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
fn inner_path_multiline() {
    assert_diag(
        ex4(),
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
}

#[test]
fn inner_path_multiline_padded() {
    assert_diag(
        ex4(),
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
            let v1 = Vtx::leaf(c1);
            let v2 = Vtx::leaf(c2);
            let v3 = Vtx::leaf(c3);
            Vtx::inner('0', vec![v1, v2, v3])
        };
        assert_diag(root, 1, diag);
    }
}

#[test]
fn final_annotation_alignment() {
    struct Ramifier;

    impl<'t> Ramify<&'t Vtx<char>> for Ramifier {
        type Key = char;

        fn children(&mut self, vtx: &'t Vtx<char>) -> impl IntoIterator<Item = &'t Vtx<char>> {
            vtx.children.iter()
        }

        fn get_key(&self, vtx: &&'t Vtx<char>) -> Self::Key {
            vtx.data
        }

        fn marker(&self, vtx: &&'t Vtx<char>) -> char {
            vtx.data
        }

        fn annotation<B: fmt::Write>(&self, vtx: &&'t Vtx<char>, mut buf: B) -> fmt::Result {
            if vtx.data == '8' {
                buf.write_str(">0\n>1\n>2")?;
            }
            Ok(())
        }
    }
    assert_diag_impl(
        ex4(),
        "\
0
├┬╮
│1├╮
││2│
│╰╮3
│ │╰╮
│ ╰╮│
├┬╮││
│4│││
5╭╯││
 6╭╯│
  7╭╯
   8 >0
     >1
     >2
",
        Ramifier,
        Config::<RoundedCorners>::new(),
    );
}
