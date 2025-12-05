use std::str::from_utf8;

use crate::writer::{DiagramWriter, RoundedCorners};

fn trs(input: &[usize]) -> Vec<((), usize)> {
    input.iter().map(|e| ((), *e)).collect()
}

#[test]
fn fork_align() {
    fn assert_fork(
        input: &[usize],
        min_index: usize,
        bounds: impl super::HalfOpen,
        output: &[usize],
        expected: &str,
        idx: usize,
    ) {
        println!("Test case: {expected}");
        let mut input_mod = trs(input);
        let output_mod = trs(output);

        let mut target: Vec<u8> = Vec::new();
        let res = super::fork_align(
            &mut DiagramWriter::<_, RoundedCorners>::new(&mut target),
            &mut input_mod,
            min_index,
            bounds,
        )
        .unwrap();

        assert_eq!(res, idx);
        assert_eq!(input_mod, output_mod);
        assert_eq!(from_utf8(&target).unwrap(), expected);
    }

    // basic left
    assert_fork(&[1, 1, 1], 0, ..2, &[0, 1, 1], "╭┤", 2);
    assert_fork(&[1, 1, 1], 2, ..2, &[0, 0, 1], "╭┤", 2);
    assert_fork(&[1, 1, 1], 2, ..3, &[0, 0, 1], "╭┤", 2);

    // post-fork re-alignment
    assert_fork(&[1, 1, 4, 4], 0, ..5, &[0, 1, 2, 2], "╭┤╭─╯", 5);
    assert_fork(&[3, 3, 5], 1, ..6, &[0, 1, 4], "╭┬─╯╭╯", 6);

    // fork right has space
    assert_fork(&[0, 0, 0, 2, 2], 0, ..3, &[0, 1, 1, 2, 2], "├╮│", 3);

    // fork right is at the end, and there is no buffer space
    assert_fork(&[0, 1, 1], 2, ..2, &[0, 1, 1], "││", 3);

    // fork right is at the end, and there is buffer space
    assert_fork(&[0, 1, 1, 1], 1, ..3, &[0, 1, 2, 2], "│├╮", 3);

    // fork middle at end
    assert_fork(&[0, 3, 3, 3, 3], 2, ..5, &[0, 1, 2, 3, 3], "│╭┬┤", 4);
    assert_fork(&[0, 4, 4, 4, 4], 2, ..5, &[0, 1, 2, 3, 3], "│╭┬┬╯", 5);
    assert_fork(&[0, 2, 2, 2, 2], 2, ..4, &[0, 1, 2, 3, 3], "│╭┼╮", 4);
    assert_fork(&[0, 1, 1, 1, 1], 3, ..4, &[0, 1, 1, 2, 3], "│├┬╮", 4);

    // fork middle, check post-alignment
    assert_fork(
        &[0, 2, 2, 2, 2, 4, 7],
        2,
        ..9,
        &[0, 1, 2, 3, 3, 4, 5],
        "│╭┼╮│╭─╯",
        8,
    );
    assert_fork(
        &[0, 2, 2, 2, 2, 5],
        2,
        ..7,
        &[0, 1, 2, 3, 3, 4],
        "│╭┼╮╭╯",
        6,
    );
    assert_fork(
        &[0, 4, 4, 4, 4, 7],
        2,
        ..8,
        &[0, 1, 2, 3, 3, 5],
        "│╭┬┬╯╭─╯",
        8,
    );

    // fork middle fails
    assert_fork(&[0, 2, 2, 2, 2], 2, ..3, &[0, 2, 2, 2, 2], "│ │", 4);
    assert_fork(&[0, 1, 1, 1, 1], 3, ..3, &[0, 1, 1, 1, 1], "││", 4);
    assert_fork(&[0, 2, 2, 2, 2, 3], 2, ..5, &[0, 2, 2, 2, 2, 4], "│ │╰╮", 5);
    assert_fork(&[0, 2, 2, 2, 2, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│╭┼╮│", 5);
    assert_fork(&[0, 1, 2, 3, 3, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│││││", 5);

    // no realignment for end bound
    assert_fork(&[0, 0, 0, 1], 1, ..2, &[0, 0, 0, 1], "││", 4);

    // print internal spaces
    assert_fork(&[0, 0, 0, 2, 3], 1, ..5, &[0, 0, 0, 2, 4], "│ │╰╮", 5);
}

#[test]
fn align() {
    fn assert_align(
        input: &[usize],
        bounds: impl super::HalfOpen,
        output: &[usize],
        expected: &str,
        idx: usize,
    ) {
        let mut input_mod = trs(input);
        let output_mod = trs(output);

        let mut target: Vec<u8> = Vec::new();
        let res = super::align(
            &mut DiagramWriter::<_, RoundedCorners>::new(&mut target),
            &mut input_mod,
            bounds,
        )
        .unwrap();
        assert_eq!(res, idx);
        assert_eq!(input_mod, output_mod);
        assert_eq!(from_utf8(&target).unwrap(), expected);
    }

    // unbounded alignment
    assert_align(&[0, 1, 1, 3, 3], 0.., &[0, 1, 1, 2, 2], "││╭╯", 4);
    assert_align(&[0, 1, 3, 6, 10], .., &[0, 1, 2, 4, 7], "││╭╯╭─╯╭──╯", 11);
    assert_align(&[], .., &[], "", 0);
    assert_align(&[1], .., &[0], "╭╯", 2);
    assert_align(&[2], .., &[0], "╭─╯", 3);
    assert_align(&[5, 5], .., &[0, 0], "╭────╯", 6);
    assert_align(&[0], .., &[0], "│", 1);

    // push right
    assert_align(&[0], 1.., &[1], "╰╮", 2);
    assert_align(&[0], 2..3, &[2], "╰─╮", 3);
    assert_align(&[0, 0], 3..4, &[3, 3], "╰──╮", 4);
    assert_align(&[0, 2, 4, 6], 3..10, &[1, 3, 5, 6], "╰╮╰╮╰╮│", 7);
    assert_align(&[0, 2, 4, 7], 3..10, &[1, 3, 5, 6], "╰╮╰╮╰╮╭╯", 8);

    // unattainable end bounds are ignored
    assert_align(&[0, 2, 2, 3], 0..1, &[0, 1, 1, 3], "│╭╯│", 4);

    // returned index could be large if a lot of space is required to satisfy the
    // alignment
    assert_align(&[0, 2, 2, 3], 3..4, &[1, 2, 2, 3], "╰╮││", 6);

    // combined
    assert_align(&[0, 3], 1..3, &[1, 2], "╰╮╭╯", 4);
}

#[test]
fn column_range() {
    let cols = &[0, 1, 1, 1, 2, 2];
    let cols_mod = trs(cols);

    for (idx, l, r) in [
        (0, 0, 1),
        (1, 1, 4),
        (2, 1, 4),
        (3, 1, 4),
        (4, 4, 6),
        (5, 4, 6),
    ] {
        assert_eq!(super::column_range(&cols_mod, idx), l..r);
    }
}
