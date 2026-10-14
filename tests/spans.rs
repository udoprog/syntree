use anyhow::{Context, Result};
use syntree::{Builder, FlavorDefault, Node, Span, Tree};

fn find<'a>(
    tree: &'a Tree<&'static str, FlavorDefault>,
    value: &str,
) -> Result<Node<'a, &'static str, FlavorDefault>> {
    tree.walk()
        .find(|n| n.value() == value)
        .with_context(|| format!("missing {value}"))
}

#[test]
fn token_with_range_queries() -> Result<()> {
    let mut b = Builder::new();
    b.open("root")?;
    b.open("x")?;
    b.token_with("a", Span::new(0, 3))?;
    b.close()?;
    b.open("y")?;
    b.token_with("b", Span::new(3, 6))?;
    b.close()?;
    b.close()?;
    let tree = b.build()?;

    let node = tree.node_with_range(1..2).context("missing 1..2")?;
    assert_eq!(node.value(), "x");
    let node = tree.node_with_range(0..3).context("missing 0..3")?;
    assert_eq!(node.value(), "x");
    let node = tree.node_with_range(4..5).context("missing 4..5")?;
    assert_eq!(node.value(), "y");
    let node = tree.node_with_range(3..3).context("missing 3..3")?;
    assert_eq!(node.value(), "y");
    let node = tree.node_with_range(2..4).context("missing 2..4")?;
    assert_eq!(node.value(), "root");
    assert!(tree.node_with_range(6..7).is_none());
    Ok(())
}

#[test]
fn token_with_matches_token_indexing() -> Result<()> {
    let mut a = Builder::new();
    a.open("root")?;
    a.token("a", 3)?;
    a.token_empty("e")?;
    a.token("b", 3)?;
    a.close()?;
    let a = a.build()?;

    let mut b = Builder::new();
    b.open("root")?;
    b.token_with("a", Span::new(0, 3))?;
    b.token_with("e", Span::new(3, 3))?;
    b.token_with("b", Span::new(3, 6))?;
    b.close()?;
    let b = b.build()?;

    assert_eq!(a, b);
    assert_eq!(a.span(), b.span());

    for (start, end) in [(0, 0), (0, 2), (2, 3), (3, 3), (3, 5), (5, 6), (2, 4)] {
        let x = a.node_with_range(start..end).map(|n| n.value());
        let y = b.node_with_range(start..end).map(|n| n.value());
        assert_eq!(x, y, "{start}..{end}");
    }

    Ok(())
}

#[test]
fn token_with_updates_tree_span() -> Result<()> {
    let mut b = Builder::new();
    b.open("root")?;
    b.open("x")?;
    b.token_with("a", Span::new(0, 3))?;
    b.close()?;
    b.open("y")?;
    b.token_with("b", Span::new(3, 6))?;
    b.close()?;
    b.close()?;
    let tree = b.build()?;
    assert_eq!(*tree.span(), Span::new(0, 6));

    let mut b = Builder::new();
    b.set_cursor(5);
    b.token("a", 3)?;
    b.token_with("b", Span::new(8, 20))?;
    let tree = b.build()?;
    assert_eq!(*tree.span(), Span::new(0, 20));
    Ok(())
}

#[test]
fn open_with_updates_tree_span() -> Result<()> {
    let mut b = Builder::new();
    b.open_with("root", Span::new(0, 10))?;
    b.close()?;
    let tree = b.build()?;
    assert_eq!(*tree.span(), Span::new(0, 10));
    assert_eq!(
        tree.first().context("missing root")?.span(),
        &Span::new(0, 10)
    );
    Ok(())
}

#[test]
fn open_with_span_not_shrunk_by_children() -> Result<()> {
    let mut b = Builder::new();
    b.open_with("child", Span::new(0, 10))?;
    b.token_with("lit", Span::new(2, 4))?;
    b.token_with("lit2", Span::new(4, 6))?;
    b.close()?;
    let tree = b.build()?;
    assert_eq!(*find(&tree, "child")?.span(), Span::new(0, 10));
    assert_eq!(*find(&tree, "lit")?.span(), Span::new(2, 4));
    assert_eq!(*find(&tree, "lit2")?.span(), Span::new(4, 6));
    assert_eq!(*tree.span(), Span::new(0, 10));

    let mut b = Builder::new();
    b.open_with("child", Span::new(0, 10))?;
    b.token_with("lit", Span::new(2, 4))?;
    b.token("lit2", 2)?;
    b.close()?;
    let tree = b.build()?;
    assert_eq!(*find(&tree, "child")?.span(), Span::new(0, 10));
    assert_eq!(*find(&tree, "lit2")?.span(), Span::new(4, 6));
    Ok(())
}

#[test]
fn open_with_grows_to_cover_children() -> Result<()> {
    let mut b = Builder::new();
    b.open("root")?;
    b.open_with("child", Span::new(2, 4))?;
    b.token_with("lit", Span::new(1, 6))?;
    b.close()?;
    b.close()?;
    let tree = b.build()?;
    assert_eq!(*find(&tree, "child")?.span(), Span::new(1, 6));
    assert_eq!(*find(&tree, "root")?.span(), Span::new(0, 6));
    Ok(())
}

#[test]
fn close_at_with_joins_into_parent() -> Result<()> {
    let mut b = Builder::new();
    b.open("root")?;
    let c = b.checkpoint()?;
    b.token("lit", 1)?;
    b.close_at_with(&c, "w", Span::new(0, 5))?;
    b.close()?;
    let tree = b.build()?;

    assert_eq!(*find(&tree, "w")?.span(), Span::new(0, 5));
    assert_eq!(*find(&tree, "root")?.span(), Span::new(0, 5));
    assert_eq!(*tree.span(), Span::new(0, 5));
    Ok(())
}

#[test]
fn close_at_with_at_root_updates_tree_span() -> Result<()> {
    let mut b = Builder::new();
    let c = b.checkpoint()?;
    b.token_with("lit", Span::new(1, 3))?;
    b.close_at_with(&c, "custom", Span::new(0, 6))?;
    b.token_with("lit2", Span::new(6, 8))?;
    let tree = b.build()?;

    assert_eq!(*find(&tree, "custom")?.span(), Span::new(0, 6));
    assert_eq!(*tree.span(), Span::new(0, 8));
    let node = tree.node_with_range(1..2).context("missing 1..2")?;
    assert_eq!(node.value(), "custom");
    let node = tree.node_with_range(7..8).context("missing 7..8")?;
    assert_eq!(node.value(), "lit2");
    Ok(())
}

#[test]
fn close_at_with_empty_checkpoint() -> Result<()> {
    let mut b = Builder::new();
    b.open("root")?;
    let c = b.checkpoint()?;
    b.close_at_with(&c, "w", Span::new(2, 5))?;
    b.close()?;
    let tree = b.build()?;

    assert_eq!(*find(&tree, "w")?.span(), Span::new(2, 5));
    assert_eq!(*find(&tree, "root")?.span(), Span::new(0, 5));
    assert_eq!(*tree.span(), Span::new(0, 5));
    Ok(())
}
