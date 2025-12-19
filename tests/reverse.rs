mod vtx;
use vtx::*;

#[test]
fn reversed_basic() {
    assert_diag(
        ex2(),
        "1\n2\n3",
        Config::new()
            .row_padding(1)
            .annotation_before_vertex(true)
            .reverse_annotation_lines(true),
        Style::rounded_corners()
            .invert()
            .gutter_width(1)
            .annotation_margin(3),
        "    3
    2
0   1
├─┴─╯
│ │ ├─╯   3
│ │ │ │   2
│ 1 │ │   1
│ │ │ │
│ │ │ │   3
│ │ │ │   2
│ │ 2 │   1
│ │ │ │
│ │ │ ├─╯   3
│ │ │ │ │   2
│ 3 │ │ │   1
│ ╰─╮ │ │
│ │ ╰─┤ │
│ │ │ │ │   3
│ │ │ │ │   2
│ │ │ 4 │   1
│ │ │ ╰─╮
│ │ │ │   3
│ │ │ │   2
│ │ 5 │   1
│ │ ╰─╮
│ │ │   3
│ │ │   2
│ 6 │   1
│ ╰─╮
│ │   3
│ │   2
7 │   1
╰─╮
│   3
│   2
8   1
",
    );
}

#[test]
fn inner_whitespace() {
    let root = {
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::leaf('2');
        let v1 = Vtx::leaf('1');
        Vtx::inner('0', vec![v3, v1, v2])
    };

    assert_diag(
        root,
        "#",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert(),
        "0 #
├┴╯
│1│ #
│ 2 #
3 #
",
    );
}

#[test]
fn reversed_no_annotation() {
    assert_diag(
        ex4(),
        "1\n2\n3",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert(),
        "  3
  2
0 1
├┴╯
││├╯ 3
││││ 2
│1││ 1
││││ 3
││││ 2
││2│ 1
│╭╯╭╯ 3
├╯╭╯│ 2
│├╯│3 1
│││││ 3
│││││ 2
│4│││ 1
│╰╮││ 3
││╰╮│ 2
5││╰╮ 1
╰╮││ 3
│╰╮│ 2
6│╰╮ 1
╰╮│ 3
│╰╮ 2
7│  1
╰╮ 3
│  2
8  1
",
    );

    assert_diag(
        ex4(),
        "1\n2\n3",
        Config::new()
            .minimize_width(true)
            .inverted_annotations(true),
        Style::rounded_corners().invert(),
        "  3
  2
0 1
├┴╯
││├╯ 3
││││ 2
│1││ 1
││││ 3
││││ 2
││2│ 1
││╰╮
││╭─╯ 3
│╭─╯│ 2
├┴╯│3 1
│││││ 3
│││││ 2
│4│││ 1
│╰╮││
││╰╮│
│││╰╮
││││ 3
││││ 2
5│││ 1
╰╮││
│╰╮│
││╰╮
│││ 3
│││ 2
6││ 1
╰╮│
│╰╮
││ 3
││ 2
7│ 1
╰╮
│ 3
│ 2
8 1
",
    );
}

#[test]
fn reversed_complex() {
    assert_diag(
        ex1(),
        "",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert(),
        "\
0
├┴╯
│1├╯
││2│
│╭╯3
├╯│╭╯
││╭╯│
│├╯││
│4││╭─╯
│││╭─╯│
││╭─╯││
│╭─╯│││
├┴╯││││
│5│││││
│ 6││││
7╰─╮│││
 8╰─╮││
  9╰─╮│
   a╰─╮
╰┴┴─╮
│b│
c╰╮
 d
",
    );

    assert_diag(
        ex8(),
        "",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert(),
        "\
0
├┴╯
│1├┴╯
│││2├┴╯
│││││3│
││4││││
│├╯││││
││5││││
││ 6│││
││╰─╮7│
││8╰─╮│
│9╰╮╰─╮
├╯│╰╮
a│││
╰╮b│
c╰┤│
││d│
││ e
│f
g
",
    );

    assert_diag(
        ex9(),
        "",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert(),
        "\
0
├┴╯
│1├┴╯
│││2├┴╯
│││││3├╯
│││││││4
││││5│││
││6│╰╮││
││ 7│╰╮│
││╰─╮│ 8
│││  9╰╮
│││╰┴╮│
││││a╰╮
││b│╰╮
│c╰╮│
│││ d
││e
│f
g
",
    );
}

#[test]
fn expand_delay() {
    let root = {
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::leaf('2');
        let v1 = Vtx::inner('1', vec![v2, v5]);
        Vtx::inner('0', vec![v3, v1, v4])
    };

    assert_diag(
        root,
        "1\n2",
        Config::new().row_padding(1).annotation_before_vertex(true),
        Style::rounded_corners().annotation_margin(3),
        "    1
0   2
├┬╮
│││   1
│1│   2
││╰╮
│├╮│
││││   1
│2││   2
│╭╯│
││╭╯   1
3││    2
╭╯│
│╭╯   1
│4    2
│
│   1
5   2
",
    );
}

#[test]
fn early_expand() {
    let root = {
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::leaf('2');
        let v1 = Vtx::inner('1', vec![v4, v5]);
        Vtx::inner('0', vec![v2, v1, v3])
    };

    assert_diag(
        root,
        "1\n2",
        Config::new().row_padding(1).annotation_before_vertex(true),
        Style::rounded_corners().annotation_margin(3),
        "    1
0   2
├┬╮
│││   1
│1│   2
│││
│││   1
2││   2
╭╯│
├╮│   1
││3   2
││
││   1
4│   2
╭╯
│   1
5   2
",
    );
}
