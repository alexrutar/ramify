use super::*;

fn assert_diag(root: Vtx<char>, expected: &str) {
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

        fn annotation<B: Write>(&self, _: &&'t Vtx<char>, mut buf: B) -> fmt::Result {
            buf.write_char('#')
        }
    }

    assert_diag_impl(
        root,
        expected,
        Ramifier,
        Config::<crate::writer::RoundedCorners>::new(),
    )
}

#[test]
fn lookahead() {
    let root = {
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::inner('2', vec![v4, v3, v5]);
        let v1 = Vtx::inner('1', vec![v6]);
        Vtx::inner('0', vec![v2, v1, v7])
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
            let v1 = Vtx::leaf(c1);
            let v2 = Vtx::leaf(c2);
            let v3 = Vtx::leaf(c3);
            Vtx::inner('0', vec![v1, v2, v3])
        };
        assert_diag(root, diag);
    }
}

#[test]
fn long_skip() {
    let root = {
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::inner('5', vec![v7]);
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::inner('2', vec![v3]);
        let v1 = Vtx::inner('1', vec![v4, v5]);
        Vtx::inner('0', vec![v1, v2, v6])
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
    assert_diag(
        ex4(),
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
    assert_diag(
        ex6(),
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
    assert_diag(
        ex5(),
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
    assert_diag(
        ex1(),
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
    assert_diag(
        ex3(),
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

#[test]
fn whitespace_after_marker() {
    assert_diag(
        ex7(),
        "\
0  #
├╮
│1  #
│├╮
││2  #
││├╮
││3│ #
│4││ #
││5│ #
│6 │ #
│├╮│
││7│ #
8│││ #
╭╯│9 #
a ││ #
├╮││
│b││ #
││c│ #
│││d #
e│││ #
 f││ #
  g│ #
   h #
",
    );

    assert_diag(
        ex8(),
        "\
0   #
├┬╮
│1├┬╮ #
│││2├┬╮ #
│││││3│ #
││4││││ #
│├╮││││
││5││││ #
││ 6│││ #
││╭─╯7│ #
││8╭─╯│ #
│9 │╭─╯ #
├╮╭╯│
a││╭╯ #
╭╯b│ #
c╭┤│ #
││d│ #
││ e #
│f #
g #
",
    );

    assert_diag(
        ex9(),
        "\
0   #
├┬╮
│1├┬╮ #
│││2├┬╮ #
│││││3├╮ #
│││││││4 #
││││5│││ #
││6│╭╯││ #
││ 7│╭╯│ #
││╭─╯│ 8 #
│││  9╭╯ #
│││╭┬╯│
││││a╭╯ #
││b│╭╯ #
│c╭╯│ #
│││ d #
││e #
│f #
g #
",
    );
}
