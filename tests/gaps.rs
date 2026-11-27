//! Gap entries: explicit spans which leave a gap after the previous token.

use anyhow::{Context, Result};
use syntree::{Builder, Error, Span};

fn spans<'a, T, F>(
    iter: impl Iterator<Item = syntree::Element<'a, T, F>>,
) -> Vec<(T, Span<F::Index>)>
where
    T: 'a + Copy,
    F: 'a + syntree::Flavor,
{
    iter.map(|e| (e.value(), e.span())).collect()
}

#[test]
fn gaps_are_invisible() -> Result<()> {
    let mut b = Builder::new();
    b.open("root")?;
    b.token("a", 2)?;
    b.token_with("b", Span::new(5, 6))?;
    b.token("c", 1)?;
    b.close()?;
    b.set_cursor(10);
    b.token("d", 2)?;
    let tree = b.build()?;

    assert_eq!(tree.len(), 5);

    let root = tree.first().and_then(|n| n.into_node()).context("root")?;
    assert_eq!(
        spans(root.children()),
        [
            ("a", Span::new(0, 2)),
            ("b", Span::new(5, 6)),
            ("c", Span::new(6, 7)),
        ]
    );
    assert_eq!(
        spans(root.children().rev()),
        [
            ("c", Span::new(6, 7)),
            ("b", Span::new(5, 6)),
            ("a", Span::new(0, 2)),
        ]
    );

    assert_eq!(
        spans(tree.walk()),
        [
            ("root", Span::new(0, 7)),
            ("a", Span::new(0, 2)),
            ("b", Span::new(5, 6)),
            ("c", Span::new(6, 7)),
            ("d", Span::new(10, 12)),
        ]
    );

    let b = root.first().and_then(|e| e.next()).context("b")?;
    assert_eq!(b.value(), "b");
    assert_eq!(b.prev().map(|e| e.value()), Some("a"));
    assert_eq!(b.next().map(|e| e.value()), Some("c"));

    let d = tree.last().context("d")?;
    assert_eq!(d.value(), "d");
    assert_eq!(d.prev().map(|e| e.value()), Some("root"));
    Ok(())
}

#[test]
fn gap_ids_are_not_tokens() -> Result<()> {
    let mut b = Builder::new();
    let a = b.token("a", 1)?;
    let c = b.token_with("c", Span::new(3, 4))?;
    let tree = b.build()?;

    assert_eq!(a.get(), 0);
    // The gap entry takes id 1.
    assert_eq!(c.get(), 2);
    assert_eq!(tree.token(a).map(|t| t.value()), Some("a"));
    assert_eq!(tree.token(c).map(|t| t.span()), Some(Span::new(3, 4)));

    let gap = syntree::pointer::PointerU32::new(1).context("pointer")?;
    assert!(tree.token(gap).is_none());
    Ok(())
}

#[test]
fn first_token_after_cursor_moved() -> Result<()> {
    let mut b = Builder::new();
    b.set_cursor(5);
    b.token("a", 3)?;
    let tree = b.build()?;

    assert_eq!(tree.len(), 1);
    assert_eq!(spans(tree.children()), [("a", Span::new(5, 8))]);
    assert_eq!(tree.node_with_range(4..5).map(|e| e.value()), None);
    assert_eq!(tree.node_with_range(5..6).map(|e| e.value()), Some("a"));
    Ok(())
}

#[test]
fn node_before_gap() -> Result<()> {
    // An empty node opened before a gapped token comes before the token,
    // walking in either direction.
    let mut b = Builder::new();
    b.token("a", 1)?;
    b.open("empty")?;
    b.close()?;
    b.token_with("b", Span::new(4, 5))?;
    let tree = b.build()?;

    assert_eq!(
        spans(tree.children()),
        [
            ("a", Span::new(0, 1)),
            ("empty", Span::point(1)),
            ("b", Span::new(4, 5)),
        ]
    );
    assert_eq!(
        spans(tree.children().rev()),
        [
            ("b", Span::new(4, 5)),
            ("empty", Span::point(1)),
            ("a", Span::new(0, 1)),
        ]
    );

    let b = tree.last().context("b")?;
    assert_eq!(b.prev().map(|e| e.value()), Some("empty"));
    Ok(())
}

#[test]
fn checkpoint_wraps_gap() -> Result<()> {
    let mut b = Builder::new();
    b.token("a", 1)?;
    let c = b.checkpoint()?;
    b.token_with("b", Span::new(3, 4))?;
    b.close_at(c, "w")?;
    let tree = b.build()?;

    let w = tree.last().and_then(|n| n.into_node()).context("w")?;
    assert_eq!(w.span(), &Span::new(3, 4));
    assert_eq!(spans(w.children()), [("b", Span::new(3, 4))]);
    assert_eq!(tree.node_with_range(1..2).map(|e| e.value()), None);
    assert_eq!(tree.node_with_range(3..4).map(|e| e.value()), Some("b"));
    Ok(())
}

#[test]
fn tokens_must_not_overlap() -> Result<()> {
    let mut b = Builder::new();
    b.token("a", 3)?;
    assert_eq!(b.token_with("b", Span::new(2, 4)), Err(Error::TokenOrder));
    assert_eq!(b.token_with("b", Span::new(4, 3)), Err(Error::TokenOrder));

    b.set_cursor(1);
    assert_eq!(b.token("b", 1), Err(Error::TokenOrder));

    let tree = b.build()?;
    assert_eq!(tree.len(), 1);
    Ok(())
}
