use crate::writer::{RoundedCorners, RoundedCornersWide, SharpCorners, SharpCornersWide};

use super::*;

fn assert_diag<B: WriteBranch>(root: Vtx<char>, config: Config<B>, expected: &str) {
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
    }

    assert_diag_impl(root, expected, Ramifier, config)
}

fn assert_diag_annot<B: WriteBranch>(root: Vtx<char>, config: Config<B>, expected: &str) {
    struct AnnotatingRamifier;

    impl<'t> Ramify<&'t Vtx<char>> for AnnotatingRamifier {
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
            write!(buf, "#")
        }
    }

    assert_diag_impl(root, expected, AnnotatingRamifier, config)
}

#[test]
fn annotation_style_rounded() {
    let config = Config::<RoundedCorners>::new();
    assert_diag(
        ex2(),
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
}

#[test]
fn annotation_style_sharp() {
    let config = Config::<SharpCorners>::new();
    assert_diag(
        ex2(),
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
}

#[test]
fn annotation_style_rounded_wide() {
    let config = Config::<RoundedCornersWide>::new();
    assert_diag(
        ex2(),
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
}

#[test]
fn annotation_style_sharp_wide() {
    let config = Config::<SharpCornersWide>::new();
    assert_diag_annot(
        ex2(),
        config,
        "\
0     #
├─┬─┐
│ 1 ├─┐ #
│ │ 2 │ #
│ 3 │ │   #
│ ┌─┘ │
│ │ ┌─┼─┐
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
    assert_diag_annot(
        ex1(),
        Config::<RoundedCorners>::new(),
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

    let config = Config::<RoundedCornersWide>::new();
    assert_diag_annot(
        ex1(),
        config,
        "\
0     #
├─┬─╮
│ 1 ├─╮ #
│ │ 2 │ #
│ ╰─╮ 3   #
│   │ ╰─╮
│   ╰─╮ │
├─┬─╮ │ │
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
      a ╭─┬─┤ #
╭───────╯ b │ #
c ╭─────────╯ #
  d #
",
    );
}

#[test]
fn slack() {
    let mut config = Config::<RoundedCorners>::new();
    config.width_slack = true;
    assert_diag(
        ex2(),
        config,
        "\
0
├┬╮
│1├╮
││2│
│3│├┬╮
│╭╯│4│
││ 5╭╯
│6╭─╯
7╭╯
 8
",
    );
}

#[test]
fn min_diag_width() {
    let mut config = Config::<RoundedCorners>::new();
    config.min_diagram_width = 10;
    assert_diag_annot(
        ex2(),
        config,
        "\
0          #
├┬╮
│1├╮       #
││2│       #
│3│├┬╮     #
│╭╯│4│     #
││ 5╭╯     #
│6╭─╯      #
7╭╯        #
 8         #
",
    );

    let mut config = Config::<RoundedCorners>::new();
    config.min_diagram_width = 4;
    config.width_slack = true;
    assert_diag_annot(
        ex2(),
        config,
        "\
0    #
├┬╮
│1├╮  #
││2│  #
│3│├┬╮ #
│╭╯│4│ #
││ 5╭╯ #
│6╭─╯ #
7╭╯  #
 8   #
",
    );
}
