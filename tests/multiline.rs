mod vtx;

use vtx::*;

fn assert_diag_annot(root: Rc<Vtx>, margin_below: usize, expected: &str) {
    let config = Config::new().row_padding(margin_below);
    let style = Style::rounded_corners();
    assert_diag(root, ">0\n>1\n>2", config, style, expected);
}

#[test]
fn multiline_annotations() {
    assert_diag_annot(
        ex3(),
        0,
        "\
0   >0
├┬╮ >1
│││ >2
│1├╮ >0
││││ >1
││││ >2
││2│ >0
││││ >1
││││ >2
│3│├╮ >0
│╭╯││ >1
││╭┤│ >2
│││4│ >0
│││╭╯ >1
││││  >2
││5│ >0
││╭╯ >1
│││  >2
│6│ >0
│╭╯ >1
││  >2
7│ >0
╭╯ >1
│  >2
8 >0
  >1
  >2
",
    );
}

#[test]
fn inner_path_multiline() {
    assert_diag_annot(
        ex4(),
        0,
        "\
0   >0
├┬╮ >1
│││ >2
│1├╮ >0
││││ >1
││││ >2
││2│ >0
││╭╯ >1
│││  >2
││3   >0
││╰─╮ >1
│╰─╮│ >2
├┬╮││
│4│││ >0
│╭╯││ >1
││╭╯│ >2
5││╭╯ >0
╭╯││  >1
│╭╯│  >2
6│╭╯ >0
╭╯│  >1
│╭╯  >2
7│ >0
╭╯ >1
│  >2
8 >0
  >1
  >2
",
    );
}

#[test]
fn inner_path_multiline_padded() {
    assert_diag_annot(
        ex4(),
        1,
        "\
0   >0
├┬╮ >1
│││ >2
│││
│1├╮ >0
││││ >1
││││ >2
││││
││2│ >0
││╭╯ >1
│││  >2
│││
││3   >0
││╰─╮ >1
│╰─╮│ >2
├┬╮││
│4│││ >0
│╭╯││ >1
││╭╯│ >2
│││╭╯
5│││ >0
╭╯││ >1
│╭╯│ >2
││╭╯
6││ >0
╭╯│ >1
│╭╯ >2
││
7│ >0
╭╯ >1
│  >2
│
8 >0
  >1
  >2
",
    );
}

#[test]
fn small_multi() {
    let expected_diags = [
        "\
0  >0
├╮ >1
││ >2
││
1│ >0
╭┤ >1
││ >2
││
2│ >0
╭╯ >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0  >0
├╮ >1
││ >2
││
1│ >0
╭┤ >1
││ >2
││
│2 >0
│  >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0   >0
├┬╮ >1
│││ >2
│││
│1│ >0
│╭╯ >1
││  >2
││
2│ >0
╭╯ >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0  >0
├╮ >1
││ >2
││
│1 >0
├╮ >1
││ >2
││
2│ >0
╭╯ >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0   >0
├┬╮ >1
│││ >2
│││
│1│ >0
│╭╯ >1
││  >2
││
│2 >0
│  >1
│  >2
│
3 >0
  >1
  >2
",
        "\
0  >0
├╮ >1
││ >2
││
│1 >0
├╮ >1
││ >2
││
│2 >0
│  >1
│  >2
│
3 >0
  >1
  >2
",
    ];

    for ((c1, c2, c3), diag) in [
        ('1', '2', '3'),
        ('1', '3', '2'),
        ('2', '1', '3'),
        ('2', '3', '1'),
        ('3', '1', '2'),
        ('3', '2', '1'),
    ]
    .into_iter()
    .zip(expected_diags)
    {
        let root = {
            let v1 = Vtx::leaf(c1);
            let v2 = Vtx::leaf(c2);
            let v3 = Vtx::leaf(c3);
            Vtx::inner('0', vec![v1, v2, v3])
        };
        assert_diag_annot(root, 1, diag);
    }
}

#[test]
fn final_annotation_alignment() {
    let root = {
        let v8 = Vtx::leaf_annotated('8', ">0\n>1\n>2");
        let v7 = Vtx::leaf('7');
        let v6 = Vtx::leaf('6');
        let v5 = Vtx::leaf('5');
        let v4 = Vtx::leaf('4');
        let v3 = Vtx::inner('3', vec![v8]);
        let v2 = Vtx::leaf('2');
        let v1 = Vtx::inner('1', vec![v7]);
        Vtx::inner('0', vec![v5, v4, v6, v1, v2, v3])
    };
    assert_diag(
        root,
        "",
        Config::new(),
        Style::rounded_corners(),
        "\
0
├┬╮
│1├╮
││2│
│╰╮3
├╮│╰╮
││╰╮│
│├╮││
│4│││
5╭╯││
 6╭╯│
  7╭╯
   8 >0
     >1
     >2
",
    );
}
