//! A custom diagram writer which adds line numbers before annotations.

use std::io;

use ramify::{
    Config, Ramify,
    writer::{Branch, DiagramWrite, MergeBranch, Style},
};

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

    fn annotate(&self, vtx: &&'t Vtx, buf: &mut String) {
        buf.push_str(&vtx.annotation);
    }
}

/// A diagram writer which modifies the `DiagramWrite` implementation of an `IOWriter` to write
/// line numbers before each annotation line and keeps track of the total number of annotations
/// written.
struct DiagWriter<W> {
    annotation_count: usize,
    inner: W,
}

impl<W> DiagWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            annotation_count: 0,
            inner,
        }
    }
}

impl<W: DiagramWrite> DiagramWrite for DiagWriter<W> {
    type Error = W::Error;

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<(), Self::Error> {
        self.inner.write_fmt(args)
    }

    fn write_branch(
        &mut self,
        start: usize,
        skip: usize,
        branch: Branch,
    ) -> Result<(), Self::Error> {
        self.inner.write_branch(start, skip, branch)
    }

    fn write_merge_branch(
        &mut self,
        start: usize,
        skip: usize,
        merge: MergeBranch,
    ) -> Result<(), Self::Error> {
        self.inner.write_merge_branch(start, skip, merge)
    }

    fn prepare_annotation(
        &mut self,
        idx: usize,
        written: usize,
        required: usize,
    ) -> Result<(), Self::Error> {
        self.inner.prepare_annotation(idx, written, required)
    }

    fn write_annotation(&mut self, idx: usize, line: &str) -> Result<(), Self::Error> {
        // increment the annotation
        self.annotation_count += 1;
        // write the line number followed by the line
        self.inner.write_fmt(format_args!("{}. {line}", idx + 1))
    }

    fn write_newline(&mut self) -> Result<(), Self::Error> {
        self.inner.write_newline()
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

    let mut writer = DiagWriter::new(Style::rounded_corners().io_writer(io::stdout().lock()));

    Config::new()
        .generator(&tree, AnnotatingRamifier)
        .write_all(&mut writer)?; // repeatedly write to stdout until the tree is empty

    // we can write into any diagram writer because of the `write_fmt` glue method
    let ct = writer.annotation_count;
    writeln!(writer, "Wrote {ct} annotation lines!")?;

    Ok(())
}
