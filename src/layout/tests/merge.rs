use crate::{Config, Generator, Ramify};
use std::{fmt, rc::Rc};

#[derive(Clone)]
struct Vtx<T> {
    data: T,
    children: Vec<Rc<Vtx<T>>>,
}

impl<T> Vtx<T> {
    fn inner(data: T, children: Vec<Rc<Vtx<T>>>) -> Rc<Self> {
        Rc::new(Self { data, children })
    }

    fn leaf(data: T) -> Rc<Self> {
        Rc::new(Self {
            data,
            children: Vec::new(),
        })
    }
}
fn assert_diag(root: Rc<Vtx<char>>, annotation: &'static str, margin_below: usize, expected: &str) {
    let mut config = Config::with_rounded_corners();
    config.row_padding = margin_below;
    assert_diag_config(root, annotation, config, expected)
}

fn assert_diag_config<B: crate::writer::WriteBranch>(
    root: Rc<Vtx<char>>,
    annotation: &'static str,
    config: Config<B>,
    expected: &str,
) {
    struct Ramifier(&'static str);

    impl Ramify<Rc<Vtx<char>>> for Ramifier {
        fn ramify(&mut self, vtx: Rc<Vtx<char>>) -> impl IntoIterator<Item = Rc<Vtx<char>>> {
            Rc::unwrap_or_clone(vtx).children
        }

        fn sort_key(&self, vtx: &Rc<Vtx<char>>) -> impl Ord {
            vtx.data
        }

        fn marker(&self, vtx: &Rc<Vtx<char>>) -> char {
            vtx.data
        }

        fn annotate<B: fmt::Write>(&self, _: &Rc<Vtx<char>>, mut buf: B) -> fmt::Result {
            buf.write_str(self.0)
        }

        fn is_identical(&self, vtx: &Rc<Vtx<char>>, other: &Rc<Vtx<char>>) -> bool {
            Rc::ptr_eq(vtx, other)
        }
    }

    println!("\nExpecting tree:\n{expected}");

    let mut writer: Vec<u8> = Vec::new();
    let mut cols = Generator::init(root, Ramifier(annotation), config);
    while cols.write_vertex(&mut writer).unwrap() {}

    let received = std::str::from_utf8(&writer).unwrap();

    println!("Got tree:\n{received}");

    assert_eq!(expected, received);
}

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
        0,
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
        0,
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

    let mut config = Config::with_rounded_corners_wide();
    config.row_padding = 1;
    config.minimize_width = true;

    assert_diag_config(
        Rc::clone(&root),
        "",
        config,
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

    crate::branch_writer! {
        #[derive(Clone)]
        pub struct MyStyle {
            charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┬", "┼"],
            gutter_width: 1,
            inverted: true,
        }
    }

    let config = Config::<MyStyle>::new();
    assert_diag_config(
        Rc::clone(&root),
        "",
        config,
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

    let config = Config::<MyStyle>::new();
    assert_diag_config(
        root,
        "#\n#",
        config,
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
│ │ │ │   #
│ │ 5 │   #
│ ╭─╯ │
├─╯ │ │
├─│─╮ │
│ │ ╰─╮ #
6 │ ╭─╯ #
│ ╭─╯ │
├─╯ │ │
│ │ │ │ #
7 │ │ │ #
╰─┤ │ │
│ │ │ │ #
8 │ │ │ #
╰─╮ ├─╮
│ ╰─╮   #
│ 9     #
│   #
a   #
",
    );
}
