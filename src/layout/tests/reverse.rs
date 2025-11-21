use super::*;

use crate::branch_writer;

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
            write!(buf, "1\n2\n3")
        }
    }

    assert_diag_impl(root, config, expected, AnnotatingRamifier);
}

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

    assert_diag_impl(root, config, expected, Ramifier);
}

fn assert_diag_impl<R: for<'t> Ramify<&'t Vtx<char>>, B: WriteBranch>(
    root: Vtx<char>,
    config: Config<B>,
    expected: &str,
    ramifier: R,
) {
    println!("\nExpecting tree:\n{expected}");

    let mut cols = Generator::init(&root, ramifier, config);

    let received = cols.branch_diagram(usize::MAX).unwrap();

    println!("Got tree:\n{received}\nReversed:");
    for line in received.lines().rev() {
        println!("{line}");
    }

    assert_eq!(expected, received);
}

#[test]
fn reversed_basic() {
    branch_writer! {
        pub struct MyStyle {
            charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┼"],
            gutter_width: 1,
            inverted: true,
        }
    }

    let mut config = Config::<MyStyle>::new();
    config.row_padding = 1;
    config.annotation_margin = 3;

    assert_diag_annot(
        ex2(),
        config,
        "    3
    2
0   1
├─┴─╯
│ │ │   3
│ │ │   2
│ 1 │   1
│ │ ├─╯
│ │ │ │   3
│ │ │ │   2
│ │ 2 │   1
│ │ │ │
│ │ │ │   3
│ │ │ │   2
│ 3 │ │   1
│ ╰─╮ │
│ │ ╰─┼─╯
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
fn reversed_no_annotation() {
    branch_writer! {
        pub struct MyStyle {
            charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┼"],
            gutter_width: 0,
            inverted: true,
        }
    }
    let mut config = Config::<MyStyle>::new();
    config.row_padding = 0;

    assert_diag_annot(
        ex4(),
        config,
        "  3
  2
0 1
├┴╯
│││ 3
│││ 2
│1│ 1
││├╯
││││ 3
││││ 2
││2│ 1
││╰╮ 3
│││  2
││3  1
││╭─╯
│╭─╯│
├┴╯││
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
}

#[test]
fn reversed_complex() {
    branch_writer! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct MyStyle {
            charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┼"],
            gutter_width: 0,
            inverted: true,
        }
    }
    let mut config = Config::<MyStyle>::new();
    config.row_padding = 0;

    assert_diag(
        ex8(),
        config.clone(),
        "\
0
├┴╯
│1│
││├┴╯
│││2│
││││├┴╯
│││││3│
││4││││
│├╯││││
││5││││
││ 6│││
││╰─╮7│
││8╰─╮│
│9 │╰─╮
├╯╰╮│
a││╰╮
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
        config,
        "\
0
├┴╯
│1│
││├┴╯
│││2│
││││├┴╯
│││││3│
││││││├╯
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
