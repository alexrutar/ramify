use std::str::from_utf8;

use crate::{
    columns::Alignment,
    layout::{
        ops::{Apply, Fork},
        tests::trs,
    },
    writer::{DiagramWriter, RoundedCorners},
};

// A convenience trait to make it easier to write tests.
trait Bounds {
    fn align(&self) -> usize;

    fn l(&self) -> usize;

    fn r(&self) -> Option<usize>;
}

impl Bounds for (usize, usize) {
    fn align(&self) -> usize {
        self.0
    }

    fn l(&self) -> usize {
        self.1
    }

    fn r(&self) -> Option<usize> {
        None
    }
}

impl Bounds for (usize, usize, usize) {
    fn align(&self) -> usize {
        self.0
    }

    fn l(&self) -> usize {
        self.1
    }

    fn r(&self) -> Option<usize> {
        Some(self.2)
    }
}

fn assert_fork(
    input: &[usize],
    minimal: &[usize],
    bounds: impl Bounds,
    output: &[usize],
    expected: &str,
    idx: usize,
    expect_isolated: bool,
) {
    println!("Case: {input:?} {expected}");
    let mut input_t = trs(input);
    let output_t = trs(output);
    let mut target: Vec<u8> = Vec::new();

    let col = Alignment {
        l: bounds.l(),
        c: input[0],
        r: bounds.r(),
        align: bounds.align(),
    };

    let mut writer = DiagramWriter::<_, RoundedCorners>::new(&mut target);

    let (x, y) = match minimal.split_first() {
        Some((fst, rest)) => (Some(*fst), rest),
        None => (None, &[][..]),
    };

    let (new, isolated) = Fork
        .apply(
            &mut writer,
            col,
            &mut input_t,
            crate::columns::ColumnIndexIter::new(x, y, 0),
        )
        .unwrap();

    assert_eq!(expected, from_utf8(&target).unwrap());
    assert_eq!(output_t, input_t);
    assert_eq!(idx, new, "alignment");
    assert_eq!(expect_isolated, isolated, "isolation");
}

#[test]
fn fork_basic() {
    // basic cases
    assert_fork(&[0, 0], &[0], (1, 0), &[1, 2], "╰┬╮", 1, true);
    assert_fork(&[0, 0], &[0], (2, 0), &[2, 3], "╰─┬╮", 1, true);
    assert_fork(&[0, 0], &[0], (2, 0, 3), &[2, 2], "╰─╮", 1, false);

    assert_fork(&[0, 0, 0], &[1], (2, 0), &[2, 3, 4], "╰─┬┬╮", 2, true);
    assert_fork(&[0, 0, 0], &[1], (2, 0, 3), &[2, 2, 2], "╰─╮", 2, false);
    assert_fork(&[0, 0, 0], &[1], (2, 0, 4), &[2, 3, 3], "╰─┬╮", 2, false);

    assert_fork(&[0, 0, 0], &[2], (2, 0), &[2, 2, 3], "╰─┬╮", 1, true);
    assert_fork(&[0, 0, 0], &[2], (2, 0, 3), &[2, 2, 2], "╰─╮", 1, false);
}

#[test]
fn fork_multi() {
    // successful multi-split
    assert_fork(
        &[0, 0, 0, 0],
        &[0, 1],
        (1, 0),
        &[1, 2, 3, 3],
        "╰┬┬╮",
        2,
        true,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[1, 2],
        (1, 0),
        &[1, 2, 3, 4],
        "╰┬┬┬╮",
        3,
        true,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[1, 3],
        (1, 0),
        &[1, 2, 3, 4],
        "╰┬┬┬╮",
        3,
        true,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[2, 3],
        (2, 0),
        &[2, 2, 3, 4],
        "╰─┬┬╮",
        2,
        true,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[0, 1, 2, 3],
        (1, 0),
        &[1, 2, 3, 4],
        "╰┬┬┬╮",
        3,
        true,
    );
}

#[test]
fn fork_insufficient() {
    // failed multi split (not enough space)
    assert_fork(
        &[0, 0, 0, 0],
        &[0, 1],
        (1, 0, 2),
        &[1, 1, 1, 1],
        "╰╮",
        2,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[0, 1],
        (1, 0, 3),
        &[1, 2, 2, 2],
        "╰┬╮",
        2,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[0, 1],
        (1, 0, 4),
        &[1, 2, 3, 3],
        "╰┬┬╮",
        2,
        true,
    );

    assert_fork(
        &[0, 0, 0, 0],
        &[1, 3],
        (1, 0, 2),
        &[1, 1, 1, 1],
        "╰╮",
        3,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[1, 3],
        (1, 0, 3),
        &[1, 2, 2, 2],
        "╰┬╮",
        3,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[1, 3],
        (1, 0, 4),
        &[1, 2, 3, 3],
        "╰┬┬╮",
        3,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[1, 3],
        (1, 0, 5),
        &[1, 2, 3, 4],
        "╰┬┬┬╮",
        3,
        true,
    );

    assert_fork(
        &[0, 0, 0, 0],
        &[2, 3],
        (2, 0, 3),
        &[2, 2, 2, 2],
        "╰─╮",
        2,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[2, 3],
        (2, 0, 4),
        &[2, 2, 3, 3],
        "╰─┬╮",
        2,
        false,
    );
    assert_fork(
        &[0, 0, 0, 0],
        &[2, 3],
        (2, 0, 5),
        &[2, 2, 3, 4],
        "╰─┬┬╮",
        2,
        true,
    );

    // left shift cases
    assert_fork(&[0, 0], &[0], (0, 0), &[0, 1], "├╮", 1, true);
    assert_fork(&[1, 1], &[0], (0, 0), &[0, 1], "╭┤", 1, true);
    assert_fork(&[3, 3], &[0], (0, 0), &[0, 1], "╭┬─╯", 1, true);

    // capped left shift
    assert_fork(
        &[2, 2, 2, 2, 2],
        &[0, 2, 4],
        (0, 0),
        &[0, 1, 2, 3, 4],
        "╭┬┼┬╮",
        4,
        true,
    );
    assert_fork(
        &[2, 2, 2, 2, 2],
        &[0, 2, 4],
        (0, 0, 1),
        &[0, 0, 0, 0, 0],
        "╭─╯",
        4,
        false,
    );
    assert_fork(
        &[2, 2, 2, 2, 2],
        &[0, 2, 4],
        (0, 0, 2),
        &[0, 1, 1, 1, 1],
        "╭┬╯",
        4,
        false,
    );
    assert_fork(
        &[2, 2, 2, 2, 2],
        &[0, 2, 4],
        (0, 0, 3),
        &[0, 1, 2, 2, 2],
        "╭┬┤",
        4,
        false,
    );
    assert_fork(
        &[2, 2, 2, 2, 2],
        &[0, 2, 4],
        (0, 0, 4),
        &[0, 1, 2, 3, 3],
        "╭┬┼╮",
        4,
        false,
    );
}

#[test]
fn fork_shift() {
    // preceding blanks are written if necessary
    assert_fork(&[3, 3], &[0], (2, 1), &[2, 3], " ╭┤", 1, true);
    assert_fork(
        &[4, 4, 4, 4],
        &[0, 2, 3],
        (2, 1),
        &[2, 3, 4, 5],
        " ╭┬┼╮",
        3,
        true,
    );

    // alignment is updated correctly
    assert_fork(&[3, 3], &[0], (0, 2), &[2, 2], "╭╯", 1, false);
    assert_fork(&[3, 3], &[], (0, 2), &[2, 2], "╭╯", 0, true);

    // do not fork prematurely; request correct amount of space
    assert_fork(
        &[1, 1, 1, 1],
        &[0, 2],
        (5, 1, 3),
        &[2, 2, 2, 2],
        "╰╮",
        3,
        false,
    );

    // same, but on the left
    assert_fork(&[3, 3, 3], &[0, 2], (0, 2), &[2, 2, 2], "╭╯", 2, false);
    assert_fork(
        &[3, 3, 3, 3],
        &[0, 2],
        (0, 2),
        &[2, 2, 2, 3],
        "╭┤",
        3,
        false,
    );

    // both clamped simultaneously; all cases
    assert_fork(&[3], &[0], (0, 2, 5), &[2], "╭╯", 0, true);
    assert_fork(&[3, 3], &[0, 1], (0, 2, 5), &[2, 2], "╭╯", 1, false);
    assert_fork(&[3, 3, 3], &[0, 2], (0, 2, 5), &[2, 2, 2], "╭╯", 2, false);
    assert_fork(
        &[3, 3, 3, 3],
        &[0, 2],
        (0, 2, 5),
        &[2, 2, 2, 3],
        "╭┤",
        3,
        false,
    );
    assert_fork(
        &[3, 3, 3, 3, 3],
        &[0, 2, 4],
        (0, 2, 5),
        &[2, 2, 2, 3, 4],
        "╭┼╮",
        4,
        false,
    );
    assert_fork(
        &[3, 3, 3, 3, 3, 3],
        &[0, 2, 4],
        (0, 2, 5),
        &[2, 2, 2, 3, 4, 4],
        "╭┼╮",
        5,
        false,
    );
}
