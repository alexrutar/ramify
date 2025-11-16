use std::str::from_utf8;

use crate::writer::DiagramWriter;

fn trs(input: &[usize]) -> Vec<((), usize)> {
    input.iter().map(|e| ((), *e)).collect()
}

#[test]
fn fork_align() {
    fn ac(
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
            &mut DiagramWriter::new(&mut target),
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
    ac(&[1, 1, 1], 0, ..2, &[0, 1, 1], "╭┤", 2);
    ac(&[1, 1, 1], 2, ..2, &[0, 0, 1], "╭┤", 2);
    ac(&[1, 1, 1], 2, ..3, &[0, 0, 1], "╭┤", 2);

    // post-fork re-alignment
    ac(&[1, 1, 4, 4], 0, ..5, &[0, 1, 2, 2], "╭┤╭─╯", 5);
    ac(&[3, 3, 5], 1, ..6, &[0, 1, 4], "╭┬─╯╭╯", 6);

    // fork right has space
    ac(&[0, 0, 0, 2, 2], 0, ..3, &[0, 1, 1, 2, 2], "├╮│", 3);

    // fork right is at the end, and there is no buffer space
    ac(&[0, 1, 1], 2, ..2, &[0, 1, 1], "││", 3);

    // fork right is at the end, and there is buffer space
    ac(&[0, 1, 1, 1], 1, ..3, &[0, 1, 2, 2], "│├╮", 3);

    // fork middle at end
    ac(&[0, 3, 3, 3, 3], 2, ..5, &[0, 1, 2, 3, 3], "│╭┬┤", 4);
    ac(&[0, 4, 4, 4, 4], 2, ..5, &[0, 1, 2, 3, 3], "│╭┬┬╯", 5);
    ac(&[0, 2, 2, 2, 2], 2, ..4, &[0, 1, 2, 3, 3], "│╭┼╮", 4);
    ac(&[0, 1, 1, 1, 1], 3, ..4, &[0, 1, 1, 2, 3], "│├┬╮", 4);

    // fork middle, check post-alignment
    ac(
        &[0, 2, 2, 2, 2, 4, 7],
        2,
        ..9,
        &[0, 1, 2, 3, 3, 4, 5],
        "│╭┼╮│╭─╯",
        8,
    );
    ac(
        &[0, 2, 2, 2, 2, 5],
        2,
        ..7,
        &[0, 1, 2, 3, 3, 4],
        "│╭┼╮╭╯",
        6,
    );
    ac(
        &[0, 4, 4, 4, 4, 7],
        2,
        ..8,
        &[0, 1, 2, 3, 3, 5],
        "│╭┬┬╯╭─╯",
        8,
    );

    // fork middle fails
    ac(&[0, 2, 2, 2, 2], 2, ..3, &[0, 2, 2, 2, 2], "│ │", 4);

    ac(&[0, 1, 1, 1, 1], 3, ..3, &[0, 1, 1, 1, 1], "││ ", 4);

    ac(&[0, 2, 2, 2, 2, 3], 2, ..5, &[0, 2, 2, 2, 2, 4], "│ │╰╮", 5);
    ac(&[0, 2, 2, 2, 2, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│╭┼╮│", 5);
    ac(&[0, 1, 2, 3, 3, 4], 2, ..5, &[0, 1, 2, 3, 3, 4], "│││││", 5);

    // no realignment for end bound
    ac(&[0, 0, 0, 1], 1, ..2, &[0, 0, 0, 1], "││", 4);

    // print internal spaces
    ac(&[0, 0, 0, 2, 3], 1, ..5, &[0, 0, 0, 2, 4], "│ │╰╮", 5);
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
        let res =
            super::align(&mut DiagramWriter::new(&mut target), &mut input_mod, bounds).unwrap();
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
