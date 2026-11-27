use anyhow::{Context, Result};
use syntree::node::Event::{Down, Next, Up};
use syntree::{Builder, Error, FlavorDefault, Span, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Call,
    Ident,
    Paren,
    Ws,
    Comment,
}

use Syntax::{Call, Comment, Ident, Paren, Root, Ws};

const SOURCE: &str = " foo (bar) // c\n";

/// ` foo (bar) // c\n` as `Root { ws Call { Ident ws Paren Ident Paren } ws
/// Comment }`.
fn sample() -> Result<Tree<Syntax, FlavorDefault>> {
    let mut b = Builder::new();
    b.open(Root)?;
    b.trivia(Ws, 1)?;
    b.open(Call)?;
    b.token(Ident, 3)?;
    b.trivia(Ws, 1)?;
    b.token(Paren, 1)?;
    b.token(Ident, 3)?;
    b.token(Paren, 1)?;
    b.close()?;
    b.trivia(Ws, 1)?;
    b.trivia(Comment, 5)?;
    b.close()?;
    Ok(b.build()?)
}

fn values<'a>(
    it: impl Iterator<Item = syntree::Element<'a, Syntax, FlavorDefault>>,
) -> Vec<Syntax> {
    it.map(|e| e.value()).collect()
}

#[test]
fn children_skip_trivia() -> Result<()> {
    let tree = sample()?;
    let root = tree.first().and_then(|e| e.into_node()).context("root")?;

    assert_eq!(values(root.children()), [Call]);
    assert_eq!(values(root.children().rev()), [Call]);
    assert_eq!(
        values(root.children().with_trivia()),
        [Ws, Call, Ws, Comment]
    );
    assert_eq!(
        values(root.children().with_trivia().rev()),
        [Comment, Ws, Call, Ws]
    );

    let call = root.children().next_node().context("call")?;
    assert_eq!(values(call.children()), [Ident, Paren, Ident, Paren]);
    assert_eq!(
        values(call.children().with_trivia()),
        [Ident, Ws, Paren, Ident, Paren]
    );

    // Both ends meet over skipped trivia.
    let mut it = call.children();
    assert_eq!(it.next().map(|e| e.value()), Some(Ident));
    assert_eq!(it.next_back().map(|e| e.value()), Some(Paren));
    assert_eq!(it.next_back().map(|e| e.value()), Some(Ident));
    assert_eq!(it.next().map(|e| e.value()), Some(Paren));
    assert!(it.next().is_none());
    assert!(it.next_back().is_none());
    Ok(())
}

#[test]
fn siblings_skip_trivia() -> Result<()> {
    let tree = sample()?;
    let call = tree
        .walk()
        .find(|e| e.value() == Call)
        .and_then(|e| e.into_node())
        .context("call")?;
    let ident = call.first().context("ident")?;

    assert_eq!(values(ident.siblings()), [Ident, Paren, Ident, Paren]);
    assert_eq!(
        values(ident.siblings().with_trivia()),
        [Ident, Ws, Paren, Ident, Paren]
    );

    // Starting on trivia skips it too.
    let ws = ident.next().context("ws")?;
    assert!(ws.is_trivia());
    assert_eq!(values(ws.siblings()), [Paren, Ident, Paren]);
    Ok(())
}

#[test]
fn point_accessors_are_structural() -> Result<()> {
    let tree = sample()?;
    let root = tree.first().and_then(|e| e.into_node()).context("root")?;

    let first = root.first().context("first")?;
    assert_eq!(first.value(), Ws);
    assert!(first.is_trivia());

    let last = root.last().context("last")?;
    assert_eq!(last.value(), Comment);
    assert!(last.as_token().is_some_and(|t| t.is_trivia()));

    assert_eq!(first.next().map(|e| e.value()), Some(Call));
    assert!(!root.children().any(|e| e.is_trivia()));
    Ok(())
}

#[test]
fn walk_skips_trivia() -> Result<()> {
    let tree = sample()?;

    assert_eq!(
        values(tree.walk()),
        [Root, Call, Ident, Paren, Ident, Paren]
    );
    assert_eq!(
        values(tree.walk().with_trivia()),
        [Root, Ws, Call, Ident, Ws, Paren, Ident, Paren, Ws, Comment]
    );

    let depths = tree
        .walk()
        .with_depths()
        .map(|(d, e)| (d, e.value()))
        .collect::<Vec<_>>();

    assert_eq!(
        depths,
        [
            (0, Root),
            (1, Call),
            (2, Ident),
            (2, Paren),
            (2, Ident),
            (2, Paren)
        ]
    );

    let depths = tree
        .walk()
        .with_trivia()
        .with_depths()
        .map(|(d, e)| (d, e.value()))
        .collect::<Vec<_>>();

    assert_eq!(
        depths,
        [
            (0, Root),
            (1, Ws),
            (1, Call),
            (2, Ident),
            (2, Ws),
            (2, Paren),
            (2, Ident),
            (2, Paren),
            (1, Ws),
            (1, Comment)
        ]
    );
    Ok(())
}

#[test]
fn walk_events_stay_balanced() -> Result<()> {
    let tree = sample()?;

    let events = tree
        .walk_events()
        .map(|(e, n)| (e, n.value()))
        .collect::<Vec<_>>();

    assert_eq!(
        events,
        [
            (Next, Root),
            (Down, Call),
            (Down, Ident),
            (Next, Paren),
            (Next, Ident),
            (Next, Paren),
            (Up, Call),
            (Up, Root),
        ]
    );

    let mut it = tree.walk_events();

    while it.next().is_some() {}

    assert_eq!(it.depth(), 0);
    Ok(())
}

/// A node whose only children are trivia is walked like an empty node.
fn trivia_only() -> Result<Tree<Syntax, FlavorDefault>> {
    let mut b = Builder::new();
    b.open(Root)?;
    b.open(Call)?;
    b.trivia(Ws, 1)?;
    b.trivia(Comment, 2)?;
    b.close()?;
    b.open(Paren)?;
    b.token(Ident, 1)?;
    b.close()?;
    b.close()?;
    Ok(b.build()?)
}

#[test]
fn trivia_only_node() -> Result<()> {
    let tree = trivia_only()?;

    let events = tree
        .walk_events()
        .map(|(e, n)| (e, n.value()))
        .collect::<Vec<_>>();

    assert_eq!(
        events,
        [
            (Next, Root),
            (Down, Call),
            (Next, Paren),
            (Down, Ident),
            (Up, Paren),
            (Up, Root),
        ]
    );

    let call = tree
        .walk()
        .find(|e| e.value() == Call)
        .and_then(|e| e.into_node())
        .context("call")?;

    assert!(call.has_children());
    assert_eq!(call.children().count(), 0);
    assert_eq!(call.children().with_trivia().count(), 2);

    // `inside` must not leak into the next sibling after skipping the
    // trivia at the end of the node.
    assert_eq!(values(call.walk().inside()), [Call]);
    assert_eq!(
        values(call.walk().with_trivia().inside()),
        [Call, Ws, Comment]
    );
    assert_eq!(values(call.walk()), [Call, Paren, Ident]);
    Ok(())
}

#[test]
fn spans_cover_trivia() -> Result<()> {
    let tree = sample()?;
    let root = tree.first().and_then(|e| e.into_node()).context("root")?;

    assert_eq!(tree.span(), &Span::new(0, 16));
    assert_eq!(root.span(), &Span::new(0, 16));

    let call = root.children().next_node().context("call")?;
    assert_eq!(call.span(), &Span::new(1, 10));

    let comment = tree.node_with_range(12..14).context("comment")?;
    assert_eq!(comment.value(), Comment);
    assert!(comment.is_trivia());
    assert_eq!(comment.span(), Span::new(11, 16));

    let ws = tree.node_with_range(4..5).context("ws")?;
    assert_eq!(ws.value(), Ws);
    Ok(())
}

#[test]
fn round_trip() -> Result<()> {
    let tree = sample()?;

    let mut out = String::new();

    for e in tree.walk().with_trivia() {
        if let Some(t) = e.as_token() {
            out.push_str(&SOURCE[t.range()]);
        }
    }

    assert_eq!(out, SOURCE);

    // Without trivia only the significant tokens remain.
    let mut out = String::new();

    for e in tree.walk() {
        if let Some(t) = e.as_token() {
            out.push_str(&SOURCE[t.range()]);
        }
    }

    assert_eq!(out, "foo(bar)");
    Ok(())
}

#[test]
fn print_includes_trivia() -> Result<()> {
    let tree = sample()?;
    let mut out = Vec::new();
    syntree::print::print_with_source(&mut out, &tree, SOURCE)?;
    let out = String::from_utf8(out)?;
    assert!(out.contains("Comment@11..16 \"// c\\n\""), "{out}");
    assert!(out.contains("Ws@0..1 \" \""), "{out}");
    Ok(())
}

#[test]
fn trivia_with_span() -> Result<()> {
    let mut b = Builder::new();
    b.open(Root)?;
    b.token_with(Ident, Span::new(0, 3))?;
    b.trivia_with(Comment, Span::new(5, 9))?;
    b.token_with(Ident, Span::new(9, 10))?;
    b.close()?;

    assert_eq!(b.trivia_with(Ws, Span::new(8, 12)), Err(Error::TokenOrder));

    let tree = b.build()?;
    let root = tree.first().and_then(|e| e.into_node()).context("root")?;

    assert_eq!(values(root.children()), [Ident, Ident]);

    let comment = root
        .children()
        .with_trivia()
        .nth(1)
        .and_then(|e| e.into_token())
        .context("comment")?;

    assert!(comment.is_trivia());
    assert_eq!(comment.span(), Span::new(5, 9));
    assert_eq!(root.span(), &Span::new(0, 10));
    Ok(())
}

#[test]
fn close_at_keeps_trivia() -> Result<()> {
    let mut b = Builder::new();
    let c = b.checkpoint()?;
    b.token(Ident, 3)?;
    b.trivia(Ws, 1)?;
    b.token(Paren, 1)?;
    b.close_at(c, Call)?;
    b.trivia(Ws, 1)?;
    let tree = b.build()?;

    let call = tree.first().and_then(|e| e.into_node()).context("call")?;
    assert_eq!(call.span(), &Span::new(0, 5));
    assert_eq!(values(call.children()), [Ident, Paren]);
    assert_eq!(values(call.children().with_trivia()), [Ident, Ws, Paren]);

    let ws = call
        .children()
        .with_trivia()
        .nth(1)
        .and_then(|e| e.into_token())
        .context("ws")?;
    assert!(ws.is_trivia());
    assert_eq!(ws.parent().map(|n| n.value()), Some(Call));

    assert_eq!(values(tree.children()), [Call]);
    assert_eq!(values(tree.children().with_trivia()), [Call, Ws]);
    Ok(())
}

#[test]
fn trivia_is_part_of_equality() -> Result<()> {
    let mut a = Builder::new();
    a.token(Ident, 1)?;
    a.trivia(Ws, 1)?;
    let a = a.build()?;

    let mut b = Builder::new();
    b.token(Ident, 1)?;
    b.token(Ws, 1)?;
    let b = b.build()?;

    let mut c = Builder::new();
    c.token(Ident, 1)?;
    let c = c.build()?;

    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_eq!(a, a.clone());
    Ok(())
}

syntree::flavor! {
    struct FlavorU8 {
        type Index = u32;
        type Width = u8;
    }
}

/// The flag shares the parent pointer, so check it at the root and under the
/// largest node id.
#[test]
fn flag_at_pointer_limits() -> Result<()> {
    let mut b: Builder<u8, FlavorU8> = Builder::new_with();

    b.trivia(0, 1)?;
    b.token(1, 1)?;

    for d in 0..125 {
        b.open(d)?;
        b.close()?;
    }

    let last = b.open(125)?;
    b.trivia(2, 1)?;
    b.token(3, 1)?;
    b.close()?;

    assert_eq!(b.open(126), Err(Error::Overflow));

    let tree = b.build()?;

    let values = |it: syntree::node::Children<'_, u8, FlavorU8>| {
        it.map(|e| (e.value(), e.is_trivia())).collect::<Vec<_>>()
    };

    assert_eq!(values(tree.children()).len(), 127);
    assert_eq!(
        &values(tree.children().with_trivia())[..2],
        [(0, true), (1, false)]
    );

    let last = tree.get(last).context("last node")?;
    assert_eq!(values(last.children()), [(3, false)]);
    assert_eq!(
        values(last.children().with_trivia()),
        [(2, true), (3, false)]
    );

    let ws = last
        .first()
        .and_then(|e| e.into_token())
        .context("trivia")?;
    assert_eq!(ws.parent().map(|n| n.id()), Some(last.id()));
    Ok(())
}

#[test]
fn debug_shows_trivia() -> Result<()> {
    let mut b = Builder::new();
    b.token(Ident, 1)?;
    b.trivia(Ws, 1)?;
    let tree = b.build()?;

    let ident = tree.first().context("ident")?;
    let ws = tree.last().context("ws")?;

    assert_eq!(format!("{ident:?}"), "Token { data: Ident, span: (0, 1) }");
    assert_eq!(
        format!("{ws:?}"),
        "Token { data: Ws, span: (1, 2), trivia: true }"
    );
    Ok(())
}
