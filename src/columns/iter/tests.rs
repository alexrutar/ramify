use std::{iter::repeat, str::from_utf8};

use crate::{
    layout::{ops::*, tests::trs},
    writer::{DiagramWriter, RoundedCorners},
};

trait Minimal {
    fn convert(self) -> Option<(usize, &'static [usize])>;
}

impl Minimal for usize {
    fn convert(self) -> Option<(usize, &'static [usize])> {
        Some((self, &[]))
    }
}

impl<const N: usize> Minimal for &'static [usize; N] {
    fn convert(self) -> Option<(usize, &'static [usize])> {
        match self.split_first() {
            Some((fst, rst)) => Some((*fst, rst)),
            None => None,
        }
    }
}

#[test]
fn col_iter() {
    fn assert_iter<const N: usize>(
        cols: &[usize],
        minimal: impl Minimal,
        expected: [(usize, std::ops::Range<usize>, Option<usize>, &[usize]); N],
    ) {
        // fix lifetimes, just for testing
        let mut active = trs(cols);
        let cl = active.clone();

        let mut col_iter = super::RawColumnIter::init(&mut active, minimal.convert());
        for (c, span, r, min) in expected {
            let (c_rec, span_rec, r_rec, mut min_rec) = col_iter.next().unwrap();
            assert_eq!(c, c_rec, "Incorrect column index");
            assert_eq!(&cl[span], span_rec, "Incorrect span");
            assert_eq!(r, r_rec, "Incorrect next index");
            for m in min {
                assert_eq!(min_rec.next().unwrap(), *m, "Incorrect min index");
            }
            assert!(min_rec.next().is_none());
        }
        assert!(col_iter.next().is_none());
    }

    // basic
    assert_iter(&[0], 0, [(0, 0..1, None, &[0])]);
    assert_iter(&[0, 0], 1, [(0, 0..2, None, &[1])]);
    assert_iter(
        &[0, 0, 1],
        2,
        [(0, 0..2, Some(1), &[]), (1, 2..3, None, &[0])],
    );

    // min index
    assert_iter(
        &[0, 1, 2, 2, 2, 2],
        &[1, 2, 5],
        [
            (0, 0..1, Some(1), &[]),
            (1, 1..2, Some(2), &[0]),
            (2, 2..6, None, &[0, 3]),
        ],
    );
    assert_iter(
        &[0, 1, 2, 2, 2, 2],
        &[1, 4, 5],
        [
            (0, 0..1, Some(1), &[]),
            (1, 1..2, Some(2), &[0]),
            (2, 2..6, None, &[2, 3]),
        ],
    );

    // large
    assert_iter(
        &[0, 0, 1, 3, 3, 3, 4, 5, 8],
        &[1, 3, 4],
        [
            (0, 0..2, Some(1), &[1][..]),
            (1, 2..3, Some(3), &[]),
            (3, 3..6, Some(4), &[0, 1]),
            (4, 6..7, Some(5), &[]),
            (5, 7..8, Some(8), &[]),
            (8, 8..9, None, &[]),
        ],
    );
}

#[derive(Clone, Copy)]
pub enum C {
    /// fork
    F,
    /// mark
    M(char),
}

fn assert_cols(
    cols: &[usize],
    minimal: impl Minimal,
    commands: impl IntoIterator<Item = C>,
    output: &[usize],
    expected: &str,
    expected_alignment: usize,
    expected_isolated: bool,
) {
    println!("Case: {cols:?} {expected}");

    let mut active = trs(cols);
    let expected_output = trs(output);
    let mut cols_mut = super::ColumnsMut::new(&mut active, minimal.convert());

    let mut target: Vec<u8> = Vec::new();
    let mut writer = DiagramWriter::<_, RoundedCorners>::new(&mut target);

    for cmd in commands {
        let ct = match cmd {
            C::F => cols_mut.apply(Fork, &mut writer).unwrap(),
            C::M(ch) => cols_mut.apply(Marker(ch), &mut writer).unwrap(),
        };

        if !ct.is_some() {
            break;
        }
    }

    let status = cols_mut.status();

    assert_eq!(expected, from_utf8(&target).unwrap());
    assert_eq!(expected_output, active);
    assert_eq!(expected_alignment, status.target_width);
    assert_eq!(expected_isolated, status.isolated);
}

#[test]
fn fork_all() {
    fn assert_cols_fork(
        cols: &[usize],
        minimal: impl Minimal,
        output: &[usize],
        expected: &str,
        expected_alignment: usize,
        expected_isolated: bool,
    ) {
        assert_cols(
            cols,
            minimal,
            repeat(C::F),
            output,
            expected,
            expected_alignment,
            expected_isolated,
        )
    }

    // basic
    assert_cols_fork(&[1, 1, 1], 0, &[0, 1, 1], "╭┤", 2, true);
    assert_cols_fork(&[1, 1, 1], 2, &[0, 0, 1], "╭┤", 2, true);
    assert_cols_fork(&[1, 1, 1], 2, &[0, 0, 1], "╭┤", 2, true);

    // post-fork re-alignment
    assert_cols_fork(&[1, 1, 4, 4], 0, &[0, 1, 2, 2], "╭┤╭─╯", 3, true);
    assert_cols_fork(&[3, 3, 5], 1, &[0, 1, 4], "╭┬─╯╭╯", 3, true);

    // fork right has space
    assert_cols_fork(&[0, 0, 0, 2, 2], 0, &[0, 1, 1, 2, 2], "├╮│", 3, true);

    // fork right is at the end
    assert_cols_fork(&[0, 1, 1], 2, &[0, 1, 2], "│├╮", 3, true);

    // fork at the end, but there is internal space
    assert_cols_fork(&[0, 2, 3, 3], 2, &[0, 1, 3, 3], "│╭╯│", 4, false);
    assert_cols_fork(&[0, 1, 3, 3], 2, &[0, 1, 2, 3], "││╭┤", 4, true);

    // fork middle at end
    assert_cols_fork(&[0, 3, 3, 3, 3], 2, &[0, 1, 2, 3, 3], "│╭┬┤", 4, true);
    assert_cols_fork(&[0, 4, 4, 4, 4], 2, &[0, 1, 2, 3, 3], "│╭┬┬╯", 4, true);
    assert_cols_fork(&[0, 2, 2, 2, 2], 2, &[0, 1, 2, 3, 3], "│╭┼╮", 4, true);
    assert_cols_fork(&[0, 1, 1, 1, 1], 3, &[0, 1, 1, 2, 3], "│├┬╮", 4, true);

    // fork middle interior
    assert_cols_fork(
        &[0, 2, 2, 2, 2, 4, 7],
        2,
        &[0, 1, 2, 3, 3, 4, 5],
        "│╭┼╮│╭─╯",
        6,
        true,
    );
    assert_cols_fork(
        &[0, 2, 2, 2, 2, 5],
        2,
        &[0, 1, 2, 3, 3, 4],
        "│╭┼╮╭╯",
        5,
        true,
    );
    assert_cols_fork(
        &[0, 4, 4, 4, 4, 7],
        2,
        &[0, 1, 2, 3, 3, 5],
        "│╭┬┬╯╭─╯",
        5,
        true,
    );

    // fork middle without space
    assert_cols_fork(
        &[0, 2, 2, 2, 2, 3],
        2,
        &[0, 1, 2, 2, 2, 4],
        "│╭┤╰╮",
        5,
        false,
    );
    assert_cols_fork(
        &[0, 1, 2, 2, 2, 4],
        2,
        &[0, 1, 2, 3, 3, 4],
        "││├╮│",
        5,
        true,
    );
}

#[test]
fn fork_marker() {
    // basic
    assert_cols(
        &[0, 1, 2],
        2,
        [C::F, C::M('*'), C::F],
        &[0, 1, 2],
        "│*│",
        3,
        true,
    );

    // no fork at marker
    assert_cols(
        &[0, 1, 1, 2],
        1,
        [C::F, C::M('*'), C::F],
        &[0, 1, 1, 3],
        "│*╰╮",
        4,
        false,
    );

    // preceding whitespace
    assert_cols(
        &[0, 2, 2, 3],
        1,
        [C::F, C::M('*'), C::F],
        &[0, 2, 2, 3],
        "│ *│",
        4,
        false,
    );

    // blocks expansion
    assert_cols(
        &[0, 0, 0, 0, 2, 2, 3, 3],
        &[1, 3, 6],
        [C::F, C::M('*'), C::F],
        &[0, 1, 1, 1, 2, 2, 5, 6],
        "├╮*╰─┬╮",
        7,
        false,
    );
    assert_cols(
        &[0, 1, 1, 1, 2, 2, 5, 6],
        &[1, 3, 6],
        [C::F, C::F, C::M('*'), C::F, C::F],
        &[0, 1, 1, 1, 2, 2, 5, 6],
        "││*  ││",
        7,
        false,
    );
    assert_cols(
        &[0, 1, 1, 1, 2, 2, 5, 6],
        &[1, 3, 6],
        repeat(C::F),
        &[0, 1, 1, 1, 4, 4, 5, 6],
        "││╰─╮││",
        7,
        false,
    );
    assert_cols(
        &[0, 1, 1, 1, 4, 4, 5, 6],
        &[1, 3, 6],
        repeat(C::F),
        &[0, 1, 2, 3, 4, 4, 5, 6],
        "│├┬╮│││",
        7,
        true,
    );
}

#[test]
fn shimmed() {
    fn assert_shim(
        cols: &[usize],
        minimal: impl Minimal,
        (col, ch): (usize, char),
        output: &[usize],
        expected: &str,
        expected_alignment: usize,
        expected_isolated: bool,
    ) {
        println!("Case: {cols:?} {expected}");

        let mut active = trs(cols);
        let expected_output = trs(output);
        let mut cols_mut = super::ColumnsMut::new(&mut active, minimal.convert())
            .with_shim((col, crate::layout::ops::Marker(ch)));

        let mut target: Vec<u8> = Vec::new();
        let mut writer = DiagramWriter::<_, RoundedCorners>::new(&mut target);

        while cols_mut.apply(Fork, &mut writer).unwrap().is_some() {}

        let status = cols_mut.cols().status();

        assert_eq!(expected, from_utf8(&target).unwrap());
        assert_eq!(expected_output, active);
        assert_eq!(expected_alignment, status.target_width);
        assert_eq!(expected_isolated, status.isolated);
    }
    // basic
    assert_shim(&[0], 0, (1, '*'), &[0], "│*", 1, true);

    // correct whitespace handling
    assert_shim(&[1], 0, (3, '*'), &[0], "╭╯ *", 1, true);

    // delays fork
    assert_shim(&[0, 2, 2], 1, (1, '*'), &[0, 2, 2], "│*│", 3, false);
    assert_cols(&[0, 2, 2], 1, [C::F, C::F], &[0, 1, 2], "│╭┤", 3, true);
    assert_shim(&[0, 3, 3, 3], 2, (1, '*'), &[0, 2, 2, 3], "│*╭┤", 4, false);

    // does not contribute to alignment
    assert_shim(
        &[0, 0, 0, 0, 3, 3],
        &[0, 2, 4],
        (2, '*'),
        &[0, 1, 1, 1, 4, 5],
        "├╮*╰┬╮",
        6,
        false,
    );
    assert_shim(
        &[0, 3, 3, 3, 3, 3],
        &[1, 2, 3, 4, 5],
        (2, '*'),
        &[0, 3, 3, 3, 4, 5],
        "│ *├┬╮",
        6,
        false,
    );

    // empty
    assert_shim(&[], &[], (2, '*'), &[], "  *", 0, true);
}
