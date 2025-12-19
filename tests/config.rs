pub mod vtx;
use vtx::*;

#[test]
fn annotation_style_rounded() {
    assert_diag(
        ex2(),
        "",
        Config::new(),
        Style::rounded_corners(),
        "\
0
├┬╮
│1├╮
││2│
│3│├╮
│╭╯││
││╭┤│
│││4│
││5╭╯
│6╭╯
7╭╯
 8
",
    );
}

#[test]
fn annotation_style_sharp() {
    assert_diag(
        ex2(),
        "",
        Config::new(),
        Style::sharp_corners(),
        "\
0
├┬┐
│1├┐
││2│
│3│├┐
│┌┘││
││┌┤│
│││4│
││5┌┘
│6┌┘
7┌┘
 8
",
    );
}

#[test]
fn annotation_style_rounded_wide() {
    assert_diag(
        ex2(),
        "",
        Config::new(),
        Style::rounded_corners().gutter_width(1),
        "\
0
├─┬─╮
│ 1 ├─╮
│ │ 2 │
│ 3 │ ├─╮
│ ╭─╯ │ │
│ │ ╭─┤ │
│ │ │ 4 │
│ │ 5 ╭─╯
│ 6 ╭─╯
7 ╭─╯
  8
",
    );
}

#[test]
fn annotation_style_sharp_wide() {
    assert_diag(
        ex2(),
        "#",
        Config::new(),
        Style::sharp_corners().gutter_width(1),
        "\
0     #
├─┬─┐
│ 1 ├─┐ #
│ │ 2 │ #
│ 3 │ ├─┐ #
│ ┌─┘ │ │
│ │ ┌─┤ │
│ │ │ 4 │ #
│ │ 5 ┌─┘ #
│ 6 ┌─┘ #
7 ┌─┘ #
  8 #
",
    );
}

#[test]
fn annotation_reported_line_width() {
    assert_diag(
        ex1(),
        "#",
        Config::new(),
        Style::rounded_corners(),
        "\
0   #
├┬╮
│1├╮ #
││2│ #
│╰╮3  #
├╮│╰╮
││╰╮│
│├╮││
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
   a╭─╯ #
╭┬┬─╯
│b│ #
c╭╯ #
 d #
",
    );

    assert_diag(
        ex1(),
        "#",
        Config::new(),
        Style::rounded_corners().gutter_width(1),
        "\
0     #
├─┬─╮
│ 1 ├─╮ #
│ │ 2 │ #
│ ╰─╮ 3   #
├─╮ │ ╰─╮
│ │ ╰─╮ │
│ ├─╮ │ │
│ 4 │ │ ╰───╮ #
│ │ │ ╰───╮ │
│ │ ╰───╮ │ │
│ ╰───╮ │ │ │
├─┬─╮ │ │ │ │
│ 5 │ │ │ │ │ #
│   6 │ │ │ │ #
7 ╭───╯ │ │ │ #
  8 ╭───╯ │ │ #
    9 ╭───╯ │ #
      a ╭───╯ #
╭─┬─┬───╯
│ b │ #
c ╭─╯ #
  d #
",
    );
}

#[test]
fn min_diag_width() {
    assert_diag(
        ex2(),
        "#",
        Config::new(),
        Style::rounded_corners().annotation_justification(11),
        "\
0          #
├┬╮
│1├╮       #
││2│       #
│3│├╮      #
│╭╯││
││╭┤│
│││4│      #
││5╭╯      #
│6╭╯       #
7╭╯        #
 8         #
",
    );
}

#[test]
fn no_inner_ws() {
    assert_diag(
        ex3(),
        "#",
        Config::new().minimize_width(true),
        Style::rounded_corners(),
        "\
0   #
├┬╮
│1├╮ #
││2│ #
│3│├╮ #
│╭╯││
││╭┤│
│││4│ #
│││╭╯
││5│ #
││╭╯
│6│ #
│╭╯
7│ #
╭╯
8 #
",
    );
}
