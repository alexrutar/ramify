use super::*;
use crate::writer::branch_writer;

fn assert_diag_annot<B: WriteBranch>(root: Vtx<char>, config: Config<B>, expected: &str) {
    struct AnnotatingRamifier;

    impl<'t> Ramify<&'t Vtx<char>> for AnnotatingRamifier {
        type Key = char;

        fn children(&mut self, vtx: &'t Vtx<char>) -> impl IntoIterator<Item = &'t Vtx<char>> {
            vtx.children.iter()
        }

        fn get_key(&self, vtx: &&'t Vtx<char>) -> Self::Key {
            vtx.data
        }

        fn marker(&self, vtx: &&'t Vtx<char>) -> char {
            vtx.data
        }

        fn annotation<B: fmt::Write>(&self, _: &&'t Vtx<char>, mut buf: B) -> fmt::Result {
            write!(buf, "#")
        }
    }

    assert_diag_impl(root, expected, AnnotatingRamifier, config)
}

#[test]
fn branch_macro() {
    branch_writer!(
        /// A weird style
        struct MyStyle {
            charset: ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"],
            gutter_width: 3,
        }
    );

    assert_diag_annot(
        ex1(),
        Config::<MyStyle>::new(),
        "\
0         #
hbbbibbbc
a   1   hbbbc #
a   a   2   a #
a   fbbbc   3     #
a       a   fbbbc
a       fbbbc   a
hbbbibbbc   a   a
a   4   a   a   fbbbbbbbc #
a   a   a   fbbbbbbbc   a
a   a   fbbbbbbbc   a   a
a   fbbbbbbbc   a   a   a
hbbbibbbc   a   a   a   a
a   5   a   a   a   a   a #
a       6   a   a   a   a #
7   dbbbbbbbe   a   a   a #
    8   dbbbbbbbe   a   a #
        9   dbbbbbbbe   a #
            a   dbbbibbbg #
dbbbbbbbbbbbbbbbe   b   a #
c   dbbbbbbbbbbbbbbbbbbbe #
    d #
",
    );
}
