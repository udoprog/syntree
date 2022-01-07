use syntree::{print, TreeBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint()?;
    tree.open("child")?;
    tree.token("lit", 3)?;
    tree.close()?;
    tree.close_at(c, "root")?;
    tree.token("sibling", 3)?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    let child = tree.node_with_range(0..3).ok_or("missing at 0..3")?;
    assert_eq!(*child.value(), "child");
    Ok(())
}
