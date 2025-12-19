mod vtx;
use vtx::*;

use std::rc::Rc;

#[test]
fn basic() {
    let root = {
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::inner('3', vec![Rc::clone(&v4)]);
        let v2 = Vtx::inner('2', vec![Rc::clone(&v4)]);
        let v1 = Vtx::inner('1', vec![v2, v5]);
        Vtx::inner('0', vec![v1, v3])
    };

    assert_diag(
        root,
        "",
        Config::new(),
        Style::rounded_corners(),
        "\
0
├╮
1╰╮
├╮│
2││
││3
├│╯
4│
 5
",
    )
}

#[test]
fn large() {
    let root = {
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::inner('4', vec![Rc::clone(&v6)]);
        let v3 = Vtx::inner('3', vec![Rc::clone(&v4)]);
        let v2 = Vtx::inner('2', vec![Rc::clone(&v4)]);
        let v1 = Vtx::inner('1', vec![v2, v5]);
        Vtx::inner('0', vec![v1, v6, v3])
    };

    assert_diag(
        root,
        "",
        Config::new(),
        Style::rounded_corners(),
        "\
0
├╮
1╰╮
├╮│
2│├╮
│││3
├││╯
4││
│5│
├─╯
6
",
    )
}

#[test]
fn complex() {
    let root = {
        let va = Vtx::leaf('a');
        let v9 = Vtx::leaf('9');
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::inner('6', vec![v7, v8, va]);
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::inner('4', vec![Rc::clone(&v6), Rc::clone(&v9)]);
        let v3 = Vtx::inner('3', vec![v6, Rc::clone(&v4), v5]);
        let v2 = Vtx::inner('2', vec![v3, v9]);
        let v1 = Vtx::inner('1', vec![v2]);
        Vtx::inner('0', vec![v4, v1])
    };

    assert_diag(
        Rc::clone(&root),
        "",
        Config::new().row_padding(1).minimize_width(true),
        Style::rounded_corners().gutter_width(1),
        "\
0
├─╮
│ 1
│ │
│ 2
│ ├─╮
│ 3 ╰───╮
│ ├─┬─╮ │
├─│─╯ │ │
│ │ ╭─╯ │
│ │ │ ╭─╯
4 │ │ │
│ │ │ │
│ │ 5 │
│ ╰─╮ │
├─╮ │ │
├─│─╯ │
│ │ ╭─╯
6 │ ╰─╮
│ ╰─╮ │
├─╮ │ │
7 │ │ │
╭─┤ │ │
8 │ │ │
╭─╯ ├─╯
│ ╭─╯
│ 9
│
a
",
    );

    // let mut config = Config::new();
    // config.reverse_annotation_lines = true;
    // config.annotation_before_vertex = true;
    assert_diag(
        Rc::clone(&root),
        "",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert().gutter_width(1),
        "\
0
├─╯
│ 1
│ 2
│ ├─╯
│ 3 ╭───╯
│ ├─┴─╯ │
├─│─╮ │ │
4 │ ╰─╮ │
│ │ 5 ╰─╮
│ ╭─╯ │
├─╯ │ │
├─│─╮ │
6 ╭─╯ │
├─╯ │ │
7 │ │ │
╰─┤ │ │
8 │ │ │
╰─╮ ├─╮
│   9
a
",
    );

    assert_diag(
        root,
        "#\n#",
        Config::new().inverted_annotations(true),
        Style::rounded_corners().invert().gutter_width(1),
        "  #
0 #
├─╯
│ │ #
│ 1 #
│ │ #
│ 2 #
│ ├─╯
│ │ │ #
│ 3 │ #
│ │ ╭───╯
│ ├─┴─╯ │
├─│─╮ │ │
│ │ ╰─╮ │ #
4 │ │ ╰─╮ #
│ │ │ │ #
│ │ 5 │ #
│ ╭─╯ │
├─╯ │ │
├─│─╮ │
│ │ ╰─╮ #
6 │ │   #
│ │ ╭─╯
│ ╭─╯ │
├─╯ │ │
│ │ │ │ #
7 │ │ │ #
╰─┤ │ │
│ │ │ │ #
8 │ │ │ #
╰─╮ ├─╮
│ ╰─╮ #
│ 9   #
│ #
a #
",
    );
}
