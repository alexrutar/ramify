use super::*;

struct Ramifier;

impl<'t> TryRamify<&'t Vtx<char>> for Ramifier {
    type Key = char;

    fn try_children(
        &mut self,
        vtx: &'t Vtx<char>,
    ) -> Result<impl IntoIterator<Item = &'t Vtx<char>>, &'t Vtx<char>> {
        if vtx.data == '2' {
            Err(vtx.children.iter().next().unwrap())
        } else {
            Ok(vtx.children.iter())
        }
    }

    fn get_key(&self, vtx: &&'t Vtx<char>) -> Self::Key {
        vtx.data
    }

    fn marker(&self, vtx: &&'t Vtx<char>) -> char {
        vtx.data
    }
}

#[test]
fn fallible_basic() {
    fn assert_tree(bytes: &[u8], expected: &str) {
        let received = std::str::from_utf8(bytes).unwrap();
        println!("{received}");
        assert_eq!(received, expected);
    }

    let tree = ex2();
    let mut writer: Vec<u8> = Vec::new();
    let mut cols = Generator::init(&tree, Ramifier, Config::with_rounded_corners());

    assert!(cols.try_write_next_vertex(&mut writer).is_ok());
    assert_tree(
        &writer,
        "\
0
├┬╮
",
    );

    assert!(cols.try_write_next_vertex(&mut writer).is_ok());
    assert!(cols.try_write_next_vertex(&mut writer).is_err());
    assert_tree(
        &writer,
        "\
0
├┬╮
│1├╮
",
    );
    assert!(cols.try_write_next_vertex(&mut writer).is_ok());
    assert_tree(
        &writer,
        "\
0
├┬╮
│1├╮
│3││
│╭╯│
││╭┼╮
",
    );
    while cols.try_write_next_vertex(&mut writer).unwrap() {}
    assert_tree(
        &writer,
        "\
0
├┬╮
│1├╮
│3││
│╭╯│
││╭┼╮
│││4│
││5╭╯
│6╭╯
7╭╯
 8
",
    );
}
