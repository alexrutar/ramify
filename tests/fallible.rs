mod vtx;
use vtx::*;

#[test]
fn basic() {
    assert_diag_fallible(
        ex9(),
        ['2', '3', '5', '1'],
        "",
        Config::new().row_padding(1),
        Style::rounded_corners(),
        4,
        "\
0
├┬╮
│1│
│││
│✕├╮
│╭┤│
││2│
││││
││✕├╮
││╭┤│
│││3│
│││││
│││✕│
│││╭┤
││││4
│││││
││5││
│││││
││✕││
││╭╯│
│6│╭╯
│╭╯│
││ 8
││╭╯
│9╰╮
│├╮│
││a│
││╭╯
││d
││
│e
│
g
",
    );
}
