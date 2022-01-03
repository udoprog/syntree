use syntree::TreeBuilder;

fn main() -> anyhow::Result<()> {
    let mut tree = TreeBuilder::new();

    tree.start_node("root1");

    tree.start_node("child1");
    tree.end_node()?;

    tree.start_node("child2");
    tree.end_node()?;

    tree.end_node()?;

    tree.start_node("root2");
    tree.end_node()?;

    let tree = tree.build()?;
    let mut it = tree.children();

    assert_eq!(it.next().map(|n| *n.data()), Some("root1"));
    assert_eq!(it.next().map(|n| *n.data()), Some("root2"));
    assert!(it.next().is_none());
    Ok(())
}
