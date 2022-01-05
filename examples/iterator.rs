use syntree::TreeBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = TreeBuilder::new();

    tree.open("root1");

    tree.open("child1");
    tree.close()?;

    tree.open("child2");
    tree.close()?;

    tree.close()?;

    tree.open("root2");
    tree.close()?;

    let tree = tree.build()?;
    let mut it = tree.children();

    assert_eq!(it.next().map(|n| *n.value()), Some("root1"));
    assert_eq!(it.next().map(|n| *n.value()), Some("root2"));
    assert!(it.next().is_none());
    Ok(())
}
