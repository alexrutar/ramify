//! # The writer implementation
//!
//! This module abstracts over writers with a special `WriteInner` trait describing the fundamental
//! operations that a writer must implement in order to be a DiagramWriter.
use super::{Branch, Charset, MergeBranch, Style};

/// A private trait to abstract over writing
pub trait WriteInner {
    type Error;

    fn write_char(&mut self, ch: char) -> Result<(), Self::Error>;
    fn write_gutter(&mut self) -> Result<(), Self::Error>;
    fn write_annotation(&mut self, line: &str) -> Result<(), Self::Error>;
    fn finish(&mut self) -> Result<(), Self::Error>;
    fn style(&self) -> &Style;
    fn charset(&self) -> &Charset {
        &self.style().charset
    }

    #[inline]
    fn write_blanks(&mut self, non_empty: bool, cols: usize, ch: char) -> Result<(), Self::Error> {
        // this is equivalent to:
        //   let extra_ws = if self.start { 0 } else { B::GUTTER_WIDTH };
        let extra_ws = self.style().gutter_width * (non_empty as usize);
        let ws = extra_ws + (1 + self.style().gutter_width) * cols;

        for _ in 0..ws {
            self.write_char(ch)?;
        }
        Ok(())
    }

    #[inline]
    fn write_shifts(&mut self, shift: usize) -> Result<(), Self::Error> {
        for _ in 0..shift {
            self.write_gutter()?;
            self.write_char(self.charset().horizontal)?;
        }
        Ok(())
    }

    #[inline]
    fn write_forks(&mut self, fork: usize) -> Result<(), Self::Error> {
        for _ in 0..fork {
            self.write_gutter()?;
            self.write_char(self.charset().down_and_horizontal)?;
        }
        Ok(())
    }

    #[inline]
    fn write_annotation_padding(
        &mut self,
        written: usize,
        required: usize,
    ) -> Result<(), Self::Error> {
        #[inline]
        const fn cols_to_chars(gw: usize, cols: usize) -> usize {
            let mask = (cols != 0) as usize;
            ((gw + 1) * cols).wrapping_sub(gw) * mask
        }

        // | - - | - - | - - | - - | - - | - - | - - |
        // |- written_chars       -|
        // |- required_chars                         |
        // |- annotation_left_alignment    -|
        //                          |- extra_chars  -|
        //                                            |- margin -|
        //                          |- pad -|
        //                          |- n_chars                  -|
        let written_chars = cols_to_chars(self.style().gutter_width, written);
        let required_chars = cols_to_chars(self.style().gutter_width, required);
        let pad = self
            .style()
            .annotation_justification
            .saturating_sub(written_chars);

        let extra_chars = required_chars - written_chars;

        // the number of blanks we need to write
        let n_blanks = pad.max(extra_chars + self.style().annotation_margin);

        for _ in 0..n_blanks {
            self.write_char(self.charset().space)?;
        }
        Ok(())
    }
}

/// A wrapper type which basically implements `DiagramWrite`, but with `self` methods. Used as a
/// shim for modular implementation of `DiagramWrite` without exposing `WriteInner` as a public
/// trait.
pub struct Shim<T>(pub T);

impl<T: WriteInner> Shim<&mut T> {
    #[inline]
    pub fn write_branch(self, start: usize, skip: usize, branch: Branch) -> Result<(), T::Error> {
        self.0
            .write_blanks(start != 0, skip, self.0.charset().space)?;
        match branch {
            Branch::Marker(m) => self.0.write_char(m),
            Branch::Continue => self.0.write_char(self.0.charset().vertical),
            Branch::ShiftForkLeft(shift, fork) => {
                self.0.write_char(self.0.charset().down_and_right)?;
                self.0.write_forks(fork)?;
                self.0.write_shifts(shift)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().up_and_left)
            }
            Branch::ShiftForkRight(shift, fork) => {
                self.0.write_char(self.0.charset().up_and_right)?;
                self.0.write_shifts(shift)?;
                self.0.write_forks(fork)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().down_and_left)
            }
            Branch::ForkLeft(fork) => {
                self.0.write_char(self.0.charset().down_and_right)?;
                self.0.write_forks(fork)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().vertical_and_left)
            }
            Branch::ForkRight(fork) => {
                self.0.write_char(self.0.charset().vertical_and_right)?;
                self.0.write_forks(fork)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().down_and_left)
            }
            Branch::ForkMiddle(fork_l, fork_r) => {
                self.0.write_char(self.0.charset().down_and_right)?;
                self.0.write_forks(fork_l)?;
                self.0.write_gutter()?;
                self.0
                    .write_char(self.0.charset().vertical_and_horizontal)?;
                self.0.write_forks(fork_r)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().down_and_left)
            }
            Branch::MergeStart => self.0.write_char(self.0.charset().vertical_and_right),
            Branch::ShiftForkLeftMergeStart(shift, fork) => {
                self.0.write_char(self.0.charset().down_and_right)?;
                self.0.write_forks(fork)?;
                self.0.write_shifts(shift)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().up_and_horizontal)
            }
            Branch::ShiftForkRightMergeStart(shift, fork) => {
                self.0.write_char(self.0.charset().up_and_right)?;
                self.0.write_shifts(shift)?;
                self.0.write_forks(fork)?;
                self.0.write_gutter()?;
                self.0.write_char(self.0.charset().down_and_horizontal)
            }
        }
    }

    #[inline]
    pub fn write_merge_branch(
        self,
        start: usize,
        skip: usize,
        merge: MergeBranch,
    ) -> Result<(), T::Error> {
        self.0
            .write_blanks(start != 0, skip, self.0.charset().horizontal)?;
        match merge {
            MergeBranch::Join => self.0.write_char(self.0.charset().up_and_horizontal),
            MergeBranch::Cross => {
                if self.0.style().merge_over {
                    self.0.write_char(self.0.charset().horizontal)
                } else {
                    self.0.write_char(self.0.charset().vertical)
                }
            }
            MergeBranch::End => self.0.write_char(self.0.charset().up_and_left),
        }
    }

    #[inline]
    pub fn write_annotation(
        self,
        written: usize,
        required: usize,
        line: &str,
    ) -> Result<(), T::Error> {
        self.0.write_annotation_padding(written, required)?;
        self.0.write_annotation(line)?;
        self.write_newline()
    }

    #[inline]
    pub fn write_newline(self) -> Result<(), T::Error> {
        self.0.finish()
    }
}
