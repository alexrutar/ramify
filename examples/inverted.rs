//! An example of a tree with annotations that is written in reverse.
use std::io::{self, Write as _};

use ramify::{Config, Generator, Ramify, branch_writer};

/// A basic recursive tree implementation.
struct Vtx {
    data: char,
    annotation: &'static str,
    children: Vec<Vtx>,
}

impl Vtx {
    /// A vertex with children.
    fn inner(data: char, children: Vec<Vtx>) -> Self {
        Self {
            data,
            annotation: "",
            children,
        }
    }

    /// A vertex with no children.
    fn leaf(data: char) -> Self {
        Self {
            data,
            annotation: "",
            children: Vec::new(),
        }
    }
}

/// A ramifier which writes annotations.
struct AnnotatingRamifier;

impl<'t> Ramify<&'t Vtx> for AnnotatingRamifier {
    type Key = char;

    fn children(&mut self, vtx: &'t Vtx) -> impl IntoIterator<Item = &'t Vtx> {
        vtx.children.iter()
    }

    fn get_key(&self, vtx: &&'t Vtx) -> Self::Key {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Vtx) -> char {
        vtx.data
    }

    fn annotation<B: std::fmt::Write>(&self, vtx: &&'t Vtx, mut buf: B) -> std::fmt::Result {
        buf.write_str(&vtx.annotation)
    }
}

fn main() -> io::Result<()> {
    // construct the tree
    let tree = {
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let mut v4 = Vtx::leaf('4');
        v4.annotation = "An annotation\nsplit over\nthree lines";
        let mut v3 = Vtx::leaf('3');
        v3.annotation = "Another annotation";
        let v2 = Vtx::inner('2', vec![v6]);
        let mut v1 = Vtx::inner('1', vec![v3]);
        v1.annotation = "An annotation\nwith two lines";

        Vtx::inner('0', vec![v7, v1, v2, v5, v4, v8])
    };

    // define a custom 'inverted' style with characters swapped for correct writing
    // from bottom-up
    branch_writer! {
        pub struct InvertedStyle {
            charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┼"],
            gutter_width: 0,
            inverted: true,
        }
    }

    // use our new style in the configuration
    let config = Config::<InvertedStyle>::new();

    let mut generator = Generator::init(&tree, AnnotatingRamifier, config);

    // we can't print line-by-line to standard out, since we need to know the entire
    // tree in order to print it backwards. instead we load the tree into a string buffer
    let diag = generator.branch_diagram(usize::MAX).unwrap();

    // iterate over the lines in reverse, printing them
    let mut writer = io::stdout();
    for line in diag.lines().rev() {
        writeln!(&mut writer, "{line}")?;
    }

    Ok(())
}
