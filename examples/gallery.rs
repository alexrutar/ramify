//! An example demonstrating various styles and how it looks with various graphs.

use std::{
    io::{self, Write as _},
    rc::Rc,
};

use argh::FromArgs;
use ramify::{Config, Generator, Ramify, writer::Style};

/// A basic recursive DAG.
#[derive(Clone)]
struct Vtx {
    data: char,
    annotation: &'static str,
    children: Vec<Rc<Vtx>>,
}

impl Vtx {
    /// A vertex with children.
    fn inner_annotated(data: char, children: Vec<Rc<Vtx>>, annotation: &'static str) -> Rc<Self> {
        Rc::new(Self {
            data,
            annotation,
            children,
        })
    }

    /// A vertex with children.
    fn inner(data: char, children: Vec<Rc<Vtx>>) -> Rc<Self> {
        Self::inner_annotated(data, children, "")
    }

    /// A vertex with no children.
    fn leaf_annotated(data: char, annotation: &'static str) -> Rc<Self> {
        Rc::new(Self {
            data,
            annotation,
            children: Vec::new(),
        })
    }

    /// A vertex with no children.
    fn leaf(data: char) -> Rc<Self> {
        Self::leaf_annotated(data, "")
    }
}

/// A ramifier which writes annotations.
struct AnnotatingRamifier;

impl Ramify<Rc<Vtx>> for AnnotatingRamifier {
    fn ramify(&mut self, vtx: Rc<Vtx>) -> impl IntoIterator<Item = Rc<Vtx>> {
        Rc::unwrap_or_clone(vtx).children
    }

    fn sort_key(&self, vtx: &Rc<Vtx>) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &Rc<Vtx>) -> char {
        vtx.data
    }

    fn annotate(&self, vtx: &Rc<Vtx>, buf: &mut String) {
        buf.push_str(vtx.annotation);
    }

    fn is_identical(&self, vtx: &Rc<Vtx>, other: &Rc<Vtx>) -> bool {
        Rc::ptr_eq(vtx, other)
    }
}

/// Command-line arguments for the styles example.
#[derive(FromArgs)]
struct Args {
    /// style to use: RoundedCorners, SharpCorners, or DoubledLines
    #[argh(option, short = 's', default = "String::from(\"RoundedCorners\")")]
    style: String,

    /// graph to display: simple, large, complex, wide, annotations, merge, or merge-annotations
    #[argh(option, short = 'g', default = "String::from(\"complex\")")]
    graph: String,

    /// draw the root at the bottom
    #[argh(switch, short = 'i')]
    invert: bool,

    /// draw the annotation lines before the vertex
    #[argh(switch)]
    annotation_before_vertex: bool,

    /// reverse the order of annotation lines
    #[argh(switch)]
    reverse_annotation_lines: bool,

    /// extra rows between vertices
    #[argh(option, default = "0")]
    row_padding: u16,

    /// margin between annotation and branch diagram
    #[argh(option, default = "1")]
    annotation_margin: u16,

    /// the gap between columns
    #[argh(option, default = "0")]
    gutter_width: u16,

    /// minimum left justification of annotations
    #[argh(option, default = "0")]
    annotation_justification: u16,

    /// avoid all internal whitespace
    #[argh(switch)]
    minimize_width: bool,

    /// draw horizontal merge lines on top of vertical lines
    #[argh(switch)]
    merge_over: bool,
}

/// Renders the tree with the specified style and configuration.
fn render(tree: Rc<Vtx>, config: Config, style: Style, invert: bool) -> io::Result<()> {
    let mut generator = Generator::with_config(tree, AnnotatingRamifier, config);

    if invert {
        // buffer the entire output and reverse it
        let diag = generator.branch_diagram(style.invert());

        let mut writer = io::stdout().lock();
        for line in diag.lines().rev() {
            writeln!(&mut writer, "{line}")?;
        }
    } else {
        let mut writer = style.io_writer(io::stdout().lock());
        // for normal styles, write line-by-line
        while !generator.is_empty() {
            generator = generator.write_next(&mut writer)?
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut args: Args = argh::from_env();

    // normalize args
    if args.invert {
        args.annotation_before_vertex = true;
        args.reverse_annotation_lines = true;
    };

    // Select the graph based on command-line argument
    let tree = match args.graph.to_lowercase().as_str() {
        "simple" => graph::simple(),
        "large" => graph::large(),
        "complex" => graph::complex(),
        "wide" => graph::wide(),
        "annotations" => graph::annotations(),
        "merge-annotations" => graph::merge_annotations(),
        "merge" => graph::merge(),
        _ => {
            eprintln!("Unknown graph: {}", args.graph);
            eprintln!(
                "Available graphs: simple, large, complex, wide, annotations, merge, merge-annotations"
            );
            std::process::exit(1);
        }
    };

    // Create configuration from command-line arguments
    let config = Config::new()
        .row_padding(args.row_padding)
        .minimize_width(args.minimize_width)
        .annotation_before_vertex(args.annotation_before_vertex)
        .reverse_annotation_lines(args.reverse_annotation_lines);

    let style = match args.style.to_lowercase().as_str() {
        "rounded_corners" | "roundedcorners" => Style::rounded_corners(),
        "sharp_corners" | "sharpcorners" => Style::sharp_corners(),
        "doubled_lines" | "doubledlines" => Style::doubled_lines(),
        _ => {
            eprintln!("Unknown style: {}", args.style);
            eprintln!("Available styles: RoundedCorners, SharpCorners, DoubledLines");
            std::process::exit(1);
        }
    }
    .annotation_margin(args.annotation_margin)
    .gutter_width(args.gutter_width)
    .annotation_justification(args.annotation_justification)
    .merge_over(args.merge_over);

    render(tree, config, style, args.invert)
}

/// Various graphs
mod graph {
    use super::Vtx;
    use std::rc::Rc;

    /// Returns a simple tree with basic branching.
    pub fn simple() -> Rc<Vtx> {
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::leaf('2');
        let v1 = Vtx::leaf('1');
        Vtx::inner('0', vec![v1, v2, v3])
    }

    /// Returns a complex tree with multiple levels and branches.
    pub fn large() -> Rc<Vtx> {
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::inner('2', vec![v6]);
        let v1 = Vtx::inner('1', vec![v3]);

        Vtx::inner('0', vec![v7, v1, v2, v5, v4, v8])
    }

    pub fn complex() -> Rc<Vtx> {
        let vg = Vtx::leaf('g');
        let vf = Vtx::leaf('f');
        let ve = Vtx::leaf('e');
        let vd = Vtx::leaf('d');
        let vc = Vtx::inner('c', vec![vf]);
        let vb = Vtx::leaf('b');
        let va = Vtx::leaf('a');
        let v9 = Vtx::inner('9', vec![ve, va]);
        let v8 = Vtx::inner('8', vec![vd]);
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::inner('4', vec![v8]);
        let v3 = Vtx::inner('3', vec![vb]);
        let v2 = Vtx::inner('2', vec![v7]);
        let v1 = Vtx::inner('1', vec![vc]);
        Vtx::inner('0', vec![vg, v1, v6, v2, v5, v3, v9, v4])
    }

    /// Returns a wide tree with many children at each level.
    pub fn wide() -> Rc<Vtx> {
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::leaf('3');
        let v2 = Vtx::leaf('2');
        let v1 = Vtx::leaf('1');
        Vtx::inner('0', vec![v1, v2, v3, v4, v5, v6])
    }

    /// Returns a tree with annotations demonstrating annotation rendering.
    pub fn annotations() -> Rc<Vtx> {
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf_annotated('4', "An annotation\nsplit over\nthree lines");
        let v3 = Vtx::leaf_annotated('3', "Another annotation");
        let v2 = Vtx::inner('2', vec![v6]);
        let v1 = Vtx::inner_annotated('1', vec![v3], "An annotation\nwith two lines");

        Vtx::inner('0', vec![v7, v1, v2, v5, v4, v8])
    }

    /// Returns a tree containing merges.
    pub fn merge() -> Rc<Vtx> {
        let v9 = Vtx::leaf('9');
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::inner('5', vec![v6, v7, v9]);
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::inner('3', vec![Rc::clone(&v5), Rc::clone(&v8)]);
        let v2 = Vtx::inner('2', vec![v5, Rc::clone(&v3), v4]);
        let v1 = Vtx::inner('1', vec![v2, v8]);
        Vtx::inner('0', vec![v3, v1])
    }

    /// Returns a tree containing merges and annotations.
    pub fn merge_annotations() -> Rc<Vtx> {
        let va = Vtx::leaf('a');
        let v9 = Vtx::leaf('9');
        let v8 = Vtx::leaf('8');
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::inner('6', vec![v7, v8, va]);
        let v5 = Vtx::leaf_annotated('5', "An annotation\nsplit over\nthree lines");
        let v4 = Vtx::inner('4', vec![Rc::clone(&v6), Rc::clone(&v9)]);
        let v3 = Vtx::inner_annotated('3', vec![v6, Rc::clone(&v4), v5], "An annotation");
        let v2 = Vtx::inner('2', vec![v3, v9]);
        let v1 = Vtx::inner('1', vec![v2]);
        Vtx::inner('0', vec![v4, v1])
    }
}
