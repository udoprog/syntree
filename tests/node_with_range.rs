use anyhow::Result;
use syntree::{Builder, FlavorDefault, Span, Tree};

type T = Tree<&'static str, FlavorDefault>;

fn range(tree: &T, start: usize, end: usize) -> Option<&'static str> {
    let a = tree.node_with_range(start..end).map(|n| n.value());
    let b = tree
        .node_with_span(Span::new(start as u32, end as u32))
        .map(|n| n.value());
    assert_eq!(
        a, b,
        "node_with_range and node_with_span differ at {start}..{end}"
    );
    a
}

fn mixed() -> Result<T> {
    Ok(syntree::tree! {
        ("tok", 3),
        "n" => { ("inner", 3) },
        "empty" => {}
    })
}

#[test]
fn range_not_contained_by_anything() -> Result<()> {
    let tree = mixed()?;
    assert_eq!(range(&tree, 0, 100), None);
    assert_eq!(range(&tree, 2, 4), None);
    assert_eq!(range(&tree, 0, 6), None);
    Ok(())
}

#[test]
fn smallest_containing_token() -> Result<()> {
    let tree = mixed()?;
    assert_eq!(range(&tree, 0, 1), Some("tok"));
    assert_eq!(range(&tree, 0, 3), Some("tok"));
    assert_eq!(range(&tree, 3, 4), Some("inner"));
    assert_eq!(range(&tree, 3, 6), Some("inner"));
    Ok(())
}

#[test]
fn start_after_end() -> Result<()> {
    let tree = mixed()?;
    assert_eq!(range(&tree, 4, 2), None);
    assert_eq!(range(&tree, 1, 0), None);
    Ok(())
}

#[test]
fn points_and_tree_end() -> Result<()> {
    let tree = mixed()?;
    assert_eq!(range(&tree, 0, 0), Some("tok"));
    // A point on a token boundary belongs to the token starting there.
    assert_eq!(range(&tree, 3, 3), Some("inner"));
    // Empty nodes are never found, and nothing contains a point at the end.
    assert_eq!(range(&tree, 6, 6), None);
    Ok(())
}

#[test]
fn nested_climbs_to_smallest_ancestor() -> Result<()> {
    let tree: T = syntree::tree! {
        "root" => {
            "a" => { ("x", 2), ("y", 2) },
            ("z", 2)
        }
    };

    assert_eq!(range(&tree, 0, 2), Some("x"));
    assert_eq!(range(&tree, 1, 3), Some("a"));
    assert_eq!(range(&tree, 0, 4), Some("a"));
    assert_eq!(range(&tree, 3, 5), Some("root"));
    assert_eq!(range(&tree, 4, 4), Some("z"));
    assert_eq!(range(&tree, 0, 6), Some("root"));
    assert_eq!(range(&tree, 0, 7), None);
    Ok(())
}

#[test]
fn gap_between_explicit_spans() -> Result<()> {
    let mut b = Builder::new();
    let c = b.checkpoint()?;
    b.token_with("lit", Span::new(1, 3))?;
    b.close_at_with(&c, "custom", Span::new(0, 6))?;
    b.token_with("lit2", Span::new(6, 8))?;
    let tree = b.build()?;

    assert_eq!(range(&tree, 0, 0), Some("custom"));
    assert_eq!(range(&tree, 1, 2), Some("lit"));
    assert_eq!(range(&tree, 4, 5), Some("custom"));
    assert_eq!(range(&tree, 0, 6), Some("custom"));
    assert_eq!(range(&tree, 7, 8), Some("lit2"));
    assert_eq!(range(&tree, 5, 7), None);
    Ok(())
}
