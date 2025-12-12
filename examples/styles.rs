//! An example demonstrating various styles and how it looks with various graphs.

use std::io::{self, Write as _};

use argh::FromArgs;
use ramify::writer::{DoubledLines, RoundedCorners, RoundedCornersWide, SharpCorners, WriteBranch};
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
    fn ramify(&mut self, vtx: &'t Vtx) -> impl IntoIterator<Item = &'t Vtx> {
        vtx.children.iter()
    }

    fn sort_key(&self, vtx: &&'t Vtx) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Vtx) -> char {
        vtx.data
    }

    fn annotate<B: std::fmt::Write>(&self, vtx: &&'t Vtx, mut buf: B) -> std::fmt::Result {
        buf.write_str(vtx.annotation)
    }
}

/// Command-line arguments for the styles example.
#[derive(FromArgs)]
struct Args {
    /// style to use: RoundedCorners, RoundedCornersWide, SharpCorners,
    /// SharpCornersWide, DoubledLines, or Inverted
    #[argh(option, short = 's', default = "String::from(\"RoundedCorners\")")]
    style: String,

    /// graph to display: simple, complex, wide, annotations, or narrow
    #[argh(option, short = 'g', default = "String::from(\"complex\")")]
    graph: String,

    /// extra padding (in rows) between vertices
    #[argh(option, default = "0")]
    row_padding: usize,

    /// margin (in characters) between annotation and branch diagram
    #[argh(option, default = "1")]
    annotation_margin: usize,

    /// minimum width of the diagram (in gutters)
    #[argh(option, default = "0")]
    min_diagram_width: usize,

    /// avoid all internal whitespace
    #[argh(switch)]
    minimize_width: bool,
}

/// Returns a simple tree with basic branching.
fn graph_simple() -> Vtx {
    let v3 = Vtx::leaf('3');
    let v2 = Vtx::leaf('2');
    let v1 = Vtx::leaf('1');
    Vtx::inner('0', vec![v1, v2, v3])
}

/// Returns a complex tree with multiple levels and branches.
fn graph_large() -> Vtx {
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

fn graph_complex() -> Vtx {
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
fn graph_wide() -> Vtx {
    let v6 = Vtx::leaf('6');
    let v5 = Vtx::leaf('5');
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::leaf('3');
    let v2 = Vtx::leaf('2');
    let v1 = Vtx::leaf('1');
    Vtx::inner('0', vec![v1, v2, v3, v4, v5, v6])
}

/// Returns a tree with annotations demonstrating annotation rendering.
fn graph_annotations() -> Vtx {
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
}

/// Returns a narrow tree with sequential branching.
fn graph_narrow() -> Vtx {
    let v4 = Vtx::leaf('4');
    let v3 = Vtx::inner('3', vec![v4]);
    let v2 = Vtx::inner('2', vec![v3]);
    let v1 = Vtx::inner('1', vec![v2]);
    Vtx::inner('0', vec![v1])
}

// Define the custom inverted style
branch_writer! {
    struct InvertedStyle {
        charset: ["│", "─", "╯", "╰",  "╮", "╭", "┤", "├", "┴", "┬", "┼"],
        gutter_width: 0,
        inverted: true,
    }
}

/// Renders the tree with the specified style and configuration.
fn render<B: WriteBranch>(tree: &Vtx, config: Config<()>) -> io::Result<()> {
    let config = config.reset_style::<B>();
    let mut generator = Generator::init(tree, AnnotatingRamifier, config);

    if B::INVERTED {
        // For inverted styles, we need to buffer the entire output and reverse it
        let mut diag = String::new();
        while generator.write_vertex_str(&mut diag) {}

        let mut writer = io::stdout();
        for line in diag.lines().rev() {
            writeln!(&mut writer, "{line}")?;
        }
    } else {
        // For normal styles, write line-by-line
        let mut writer = io::stdout();
        while generator.write_vertex(&mut writer)? {}
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    // Select the graph based on command-line argument
    let tree = match args.graph.to_lowercase().as_str() {
        "simple" => graph_simple(),
        "large" => graph_large(),
        "complex" => graph_complex(),
        "wide" => graph_wide(),
        "annotations" => graph_annotations(),
        "narrow" => graph_narrow(),
        _ => {
            eprintln!("Unknown graph: {}", args.graph);
            eprintln!("Available graphs: simple, large, complex, wide, annotations, narrow");
            std::process::exit(1);
        }
    };

    // Create configuration from command-line arguments
    let mut config = Config::without_style();
    config.row_padding = args.row_padding;
    config.annotation_margin = args.annotation_margin;
    config.min_diagram_width = args.min_diagram_width;
    config.minimize_width = args.minimize_width;

    // Select the style and render
    // We need to use a match here because Rust requires concrete types at compile time
    match args.style.to_lowercase().as_str() {
        "rounded_corners" | "roundedcorners" => render::<RoundedCorners>(&tree, config),
        "rounded_corners_wide" | "roundedcornerswide" => {
            render::<RoundedCornersWide>(&tree, config)
        }
        "sharp_corners" | "sharpcorners" => render::<SharpCorners>(&tree, config),
        "sharp_corners_wide" | "sharpcornerswide" => {
            render::<ramify::writer::SharpCornersWide>(&tree, config)
        }
        "doubled_lines" | "doubledlines" => render::<DoubledLines>(&tree, config),
        "inverted" => render::<InvertedStyle>(&tree, config),
        _ => {
            eprintln!("Unknown style: {}", args.style);
            eprintln!("Available styles: RoundedCorners, RoundedCornersWide, SharpCorners,");
            eprintln!("                  SharpCornersWide, DoubledLines, Inverted");
            std::process::exit(1);
        }
    }
}
