use super::*;
use crate::writer::branch_writer;

fn assert_diag_annot<B: WriteBranch>(root: Vertex<char>, config: Config<B>, expected: &str) {
    struct AnnotatingRamifier;

    impl<'t> Ramify<&'t Vertex<char>> for AnnotatingRamifier {
        type Key = char;

        fn children(&mut self, vtx: &'t Vertex<char>) -> impl Iterator<Item = &'t Vertex<char>> {
            vtx.children.iter()
        }

        fn get_key(&self, vtx: &&'t Vertex<char>) -> Self::Key {
            vtx.data
        }

        fn marker(&self, vtx: &&'t Vertex<char>) -> char {
            vtx.data
        }

        fn annotation<B: fmt::Write>(&self, _: &&'t Vertex<char>, mut buf: B) -> fmt::Result {
            write!(buf, "#")
        }
    }

    assert_diag_impl(root, expected, AnnotatingRamifier, config)
}

#[test]
fn branch_macro() {
    let root = ex1();

    branch_writer!(
        /// A weird style
        struct MyStyle {
            charset: ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"],
            gutter_width: 3,
        }
    );

    assert_diag_annot(
        root.clone(),
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
