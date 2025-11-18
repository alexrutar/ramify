use std::str::from_utf8;

use crate::writer::Writer;

fn trs(input: &[usize]) -> Vec<((), usize)> {
    input.iter().map(|e| ((), *e)).collect()
}

#[test]
fn fork_align() {
    fn ab<const FORK: bool>(
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
        let res = super::fork_align::<_, _, FORK>(
            &mut Writer::with_default_config(&mut target),
            &mut input_mod,
            min_index,
            bounds,
        )
        .unwrap();

        assert_eq!(res, idx);
        assert_eq!(input_mod, output_mod);
        assert_eq!(from_utf8(&target).unwrap(), expected);
    }

    fn a(
        input: &[usize],
        min_index: usize,
        bounds: impl super::HalfOpen,
        output: &[usize],
        expected: &str,
        idx: usize,
    ) {
        ab::<true>(input, min_index, bounds, output, expected, idx);
    }

    fn b(
        input: &[usize],
        min_index: usize,
        bounds: impl super::HalfOpen,
        output: &[usize],
        expected: &str,
        idx: usize,
    ) {
        ab::<false>(input, min_index, bounds, output, expected, idx);
    }

    // basic left
    a(&[1, 1, 1], 0, ..2, &[0, 1, 1], "╭┤", 2);
    b(&[1, 1, 1], 0, ..2, &[0, 0, 0], "╭╯", 2);

    a(&[1, 1, 1], 2, ..2, &[0, 0, 1], "╭┤", 2);
    b(&[1, 1, 1], 2, ..2, &[1, 1, 1], " │", 2);

    a(&[1, 1, 1], 2, ..3, &[0, 0, 1], "╭┤", 2);
    b(&[1, 1, 1], 2, ..3, &[1, 1, 1], " │", 2);

    // post-fork re-alignment
    a(&[1, 1, 4, 4], 0, ..5, &[0, 1, 2, 2], "╭┤╭─╯", 5);
    b(&[1, 1, 4, 4], 0, ..5, &[0, 0, 2, 2], "╭╯╭─╯", 5);

    a(&[3, 3, 5], 1, ..6, &[0, 1, 4], "╭┬─╯╭╯", 6);
    b(&[3, 3, 5], 1, ..6, &[1, 1, 4], " ╭─╯╭╯", 6);

    // fork right has space
    a(&[0, 0, 0, 2, 2], 0, ..3, &[0, 1, 1, 2, 2], "├╮│", 3);
    b(&[0, 0, 0, 2, 2], 0, ..3, &[0, 0, 0, 2, 2], "│ │", 3);

    // fork right is at the end, and there is no buffer space
    a(&[0, 1, 1], 2, ..2, &[0, 1, 1], "││", 3);
    b(&[0, 1, 1], 2, ..2, &[0, 1, 1], "││", 3);

    // fork right is at the end, and there is buffer space
    a(&[0, 1, 1, 1], 1, ..3, &[0, 1, 2, 2], "│├╮", 3);
    b(&[0, 1, 1, 1], 1, ..3, &[0, 1, 1, 1], "││ ", 3);
    b(&[0, 1, 1, 1], 3, ..3, &[0, 2, 2, 2], "│╰╮", 3);

    // fork middle at end
    a(&[0, 3, 3, 3, 3], 2, ..5, &[0, 1, 2, 3, 3], "│╭┬┤", 4);
    b(&[0, 3, 3, 3, 3], 2, ..5, &[0, 2, 2, 2, 2], "│ ╭╯", 4);

    a(&[0, 4, 4, 4, 4], 2, ..5, &[0, 1, 2, 3, 3], "│╭┬┬╯", 5);
    b(&[0, 4, 4, 4, 4], 2, ..5, &[0, 2, 2, 2, 2], "│ ╭─╯", 5);

    a(&[0, 2, 2, 2, 2], 2, ..4, &[0, 1, 2, 3, 3], "│╭┼╮", 4);
    b(&[0, 2, 2, 2, 2], 2, ..4, &[0, 2, 2, 2, 2], "│ │ ", 4);

    a(&[0, 1, 1, 1, 1], 3, ..4, &[0, 1, 1, 2, 3], "│├┬╮", 4);
    b(&[0, 1, 1, 1, 1], 3, ..4, &[0, 2, 2, 2, 2], "│╰╮ ", 4);

    // fork middle, check post-alignment
    a(
        &[0, 2, 2, 2, 2, 4, 7],
        2,
        ..9,
        &[0, 1, 2, 3, 3, 4, 5],
        "│╭┼╮│╭─╯",
        8,
    );
    a(
        &[0, 2, 2, 2, 2, 5],
        2,
        ..7,
        &[0, 1, 2, 3, 3, 4],
        "│╭┼╮╭╯",
        6,
    );
    a(
        &[0, 4, 4, 4, 4, 7],
        2,
        ..8,
        &[0, 1, 2, 3, 3, 5],
        "│╭┬┬╯╭─╯",
        8,
    );

    // fork middle fails
    a(&[0, 2, 2, 2, 2], 2, ..3, &[0, 2, 2, 2, 2], "│ │", 4);
    b(&[0, 2, 2, 2, 2], 2, ..3, &[0, 2, 2, 2, 2], "│ │", 4);

    a(&[0, 1, 1, 1, 1], 3, ..3, &[0, 1, 1, 1, 1], "││ ", 4);
    b(&[0, 1, 1, 1, 1], 3, ..3, &[0, 1, 1, 1, 1], "││ ", 4);

    a(&[0, 2, 2, 2, 2, 3], 2, ..5, &[0, 2, 2, 2, 2, 4], "│ │╰╮", 5);
    b(&[0, 2, 2, 2, 2, 3], 2, ..5, &[0, 2, 2, 2, 2, 4], "│ │╰╮", 5);
    a(&[0, 2, 2, 2, 2, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│╭┼╮│", 5);
    a(&[0, 1, 2, 3, 3, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│││││", 5);
    b(&[0, 1, 2, 3, 3, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│││││", 5);

    // no realignment for end bound
    a(&[0, 0, 0, 1], 1, ..2, &[0, 0, 0, 1], "││", 4);
    b(&[0, 0, 0, 1], 1, ..2, &[0, 0, 0, 1], "││", 4);

    // print internal spaces
    a(&[0, 0, 0, 2, 3], 1, ..5, &[0, 0, 0, 2, 4], "│ │╰╮", 5);
    b(&[0, 0, 0, 2, 3], 1, ..5, &[0, 0, 0, 2, 4], "│ │╰╮", 5);
}

#[test]
fn align() {
    fn ac(
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
            &mut Writer::with_default_config(&mut target),
            &mut input_mod,
            bounds,
        )
        .unwrap();
        assert_eq!(res, idx);
        assert_eq!(input_mod, output_mod);
        assert_eq!(from_utf8(&target).unwrap(), expected);
    }

    // unbounded alignment
    ac(&[0, 1, 1, 3, 3], 0.., &[0, 1, 1, 2, 2], "││╭╯", 4);
    ac(&[0, 1, 3, 6, 10], .., &[0, 1, 2, 4, 7], "││╭╯╭─╯╭──╯", 11);
    ac(&[], .., &[], "", 0);
    ac(&[1], .., &[0], "╭╯", 2);
    ac(&[2], .., &[0], "╭─╯", 3);
    ac(&[5, 5], .., &[0, 0], "╭────╯", 6);
    ac(&[0], .., &[0], "│", 1);

    // push right
    ac(&[0], 1.., &[1], "╰╮", 2);
    ac(&[0], 2..3, &[2], "╰─╮", 3);
    ac(&[0, 0], 3..4, &[3, 3], "╰──╮", 4);
    ac(&[0, 2, 4, 6], 3..10, &[1, 3, 5, 6], "╰╮╰╮╰╮│", 7);
    ac(&[0, 2, 4, 7], 3..10, &[1, 3, 5, 6], "╰╮╰╮╰╮╭╯", 8);

    // unattainable end bounds are ignored
    ac(&[0, 2, 2, 3], 0..1, &[0, 1, 1, 3], "│╭╯│", 4);

    // returned index could be large if a lot of space is required to satisfy the
    // alignment
    ac(&[0, 2, 2, 3], 3..4, &[1, 2, 2, 3], "╰╮││", 6);

    // combined
    ac(&[0, 3], 1..3, &[1, 2], "╰╮╭╯", 4);
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
