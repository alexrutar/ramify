//! A custom diagram writer which also logs diagram write calls

use std::{io, rc::Rc};

use ramify::{
    Generator, Ramify,
    writer::{Branch, DiagramWrite, MergeBranch, Style},
};

#[derive(Clone)]
pub struct Vtx {
    data: char,
    annotation: &'static str,
    children: Vec<Rc<Vtx>>,
}

impl Vtx {
    pub fn leaf(data: char) -> Rc<Self> {
        Self::leaf_annotated(data, "")
    }

    pub fn leaf_annotated(data: char, annotation: &'static str) -> Rc<Self> {
        Rc::new(Self {
            data,
            annotation,
            children: Vec::new(),
        })
    }

    pub fn inner(data: char, children: Vec<Rc<Vtx>>) -> Rc<Self> {
        Self::inner_annotated(data, "", children)
    }

    pub fn inner_annotated(
        data: char,
        annotation: &'static str,
        children: Vec<Rc<Vtx>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            data,
            annotation,
            children,
        })
    }
}

struct Ramifier;

impl<'t> Ramify<&'t Rc<Vtx>> for Ramifier {
    fn ramify(&mut self, vtx: &'t Rc<Vtx>) -> impl IntoIterator<Item = &'t Rc<Vtx>> {
        vtx.children.iter()
    }

    fn sort_key(&self, vtx: &&'t Rc<Vtx>) -> impl Ord {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Rc<Vtx>) -> char {
        vtx.data
    }

    fn annotate(&self, vtx: &&'t Rc<Vtx>, buf: &mut String) {
        buf.push_str(vtx.annotation);
    }

    fn is_identical(&self, vtx: &&'t Rc<Vtx>, other: &&'t Rc<Vtx>) -> bool {
        Rc::ptr_eq(vtx, other)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum Instruction {
    WriteBranch(usize, usize, Branch),
    WriteMergeBranch(usize, usize, MergeBranch),
    PrepareAnnotation(usize, usize, usize),
    WriteAnnotation(usize, String),
    WriteNewline,
}

// A diagram writer which logs the write calls to an internal buffer.
struct LoggingWriter<W> {
    calls: Vec<Instruction>,
    writer: W,
}

impl<W> LoggingWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            calls: Vec::new(),
            writer,
        }
    }

    fn write_calls(&mut self) -> Result<(), W::Error>
    where
        W: DiagramWrite,
    {
        writeln!(self.writer, "\nTree built by the following calls:")?;
        for instr in &self.calls {
            writeln!(self.writer, "* {instr:?}")?;
        }
        Ok(())
    }
}

impl<W: DiagramWrite> DiagramWrite for LoggingWriter<W> {
    type Error = W::Error;

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<(), Self::Error> {
        // don't log a fmt call since this is not emitted by a Generator anyway
        self.writer.write_fmt(args)
    }

    fn write_branch(
        &mut self,
        start: usize,
        skip: usize,
        branch: Branch,
    ) -> Result<(), Self::Error> {
        self.calls
            .push(Instruction::WriteBranch(start, skip, branch));
        self.writer.write_branch(start, skip, branch)
    }

    fn write_merge_branch(
        &mut self,
        start: usize,
        skip: usize,
        merge: MergeBranch,
    ) -> Result<(), Self::Error> {
        self.calls
            .push(Instruction::WriteMergeBranch(start, skip, merge));
        self.writer.write_merge_branch(start, skip, merge)
    }

    fn prepare_annotation(
        &mut self,
        idx: usize,
        written: usize,
        required: usize,
    ) -> Result<(), Self::Error> {
        self.calls
            .push(Instruction::PrepareAnnotation(idx, written, required));
        self.writer.prepare_annotation(idx, written, required)
    }

    fn write_annotation(&mut self, idx: usize, line: &str) -> Result<(), Self::Error> {
        self.calls
            .push(Instruction::WriteAnnotation(idx, line.to_owned()));
        self.writer.write_annotation(idx, line)
    }

    fn write_newline(&mut self) -> Result<(), Self::Error> {
        self.calls.push(Instruction::WriteNewline);
        self.writer.write_newline()
    }
}

fn main() -> io::Result<()> {
    // modify this example to see how the instruction calls change.
    let root = {
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf_annotated('4', "L0");
        let v3 = Vtx::inner('3', vec![Rc::clone(&v4)]);
        let v2 = Vtx::inner_annotated('2', "L0", vec![Rc::clone(&v4)]);
        let v1 = Vtx::inner('1', vec![v2, v5]);
        Vtx::inner_annotated('0', "L0\nL1", vec![v1, v3])
    };

    let mut writer = LoggingWriter::new(Style::rounded_corners().io_writer(io::stdout().lock()));
    Generator::new(&root, Ramifier).write_all(&mut writer)?;

    writer.write_calls()
}
