use anyhow::{Context, Result};
use syntree::{Error, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Number,
    Lit,
    Whitespace,
}

use Syntax::{Lit, Number, Root, Whitespace};

#[test]
fn balanced_checkpoint() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let c = tree.checkpoint()?;

    tree.open(Number)?;
    tree.token(Lit, 2)?;
    tree.close()?;

    tree.token(Whitespace, 3)?;

    tree.open(Number)?;
    tree.token(Lit, 2)?;
    tree.token(Lit, 2)?;
    tree.close()?;

    tree.close_at(&c, Root)?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        Root => {
            Number => {
                (Lit, 2)
            },
            (Whitespace, 3),
            Number => {
                (Lit, 2),
                (Lit, 2)
            }
        }
    };

    assert_eq!(expected, tree);
    Ok(())
}

#[test]
fn test_checkpoint_mutation() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let outer = tree.checkpoint()?;
    let inner = tree.checkpoint()?;
    tree.token("b", 3)?;
    tree.close_at(&inner, "operation")?;
    tree.close_at(&outer, "operation")?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        "operation" => {
            "operation" => {
                ("b", 3)
            }
        }
    };

    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn test_nested_checkpoints() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let a = tree.checkpoint()?;
    tree.token("a", 3)?;
    let b = tree.checkpoint()?;
    tree.token("b", 3)?;
    tree.close_at(&b, "operation")?;
    tree.close_at(&a, "operation")?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        "operation" => {
            ("a", 3),
            "operation" => {
                ("b", 3)
            }
        }
    };

    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn test_nested_checkpoints2() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let a = tree.checkpoint()?;
    let b = tree.checkpoint()?;
    tree.token("b", 3)?;
    tree.close_at(&b, "operation")?;
    tree.token("a", 3)?;
    tree.close_at(&a, "operation")?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        "operation" => {
            "operation" => {
                ("b", 3)
            },
            ("a", 3)
        }
    };

    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn test_checkpoints_around_close() -> Result<()> {
    let mut tree = syntree::Builder::new();

    tree.open("operation")?;
    tree.token("b", 3)?;
    let _ = tree.checkpoint()?; // this line *shouldn't* affect anything but it triggers the bug
    tree.close()?;
    let b = tree.checkpoint()?;
    tree.token("a", 3)?;
    tree.close_at(&b, "operation")?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        "operation" => {
            ("b", 3)
        },
        "operation" => {
            ("a", 3)
        }
    };

    assert_eq!(tree, expected);
    Ok(())
}

fn values<'a, T, F, I>(nodes: I) -> Vec<T>
where
    T: 'a + Copy,
    F: 'a + syntree::Flavor,
    I: Iterator<Item = syntree::Node<'a, T, F>>,
{
    nodes.map(|n| n.value()).collect()
}

#[test]
fn close_at_root_updates_last() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let c = tree.checkpoint()?;
    tree.token("a", 1)?;
    tree.token("b", 1)?;
    let w = tree.close_at(&c, "w")?;

    let tree = tree.build()?;

    assert_eq!(tree.last().map(|n| n.id()), Some(w));
    assert_eq!(values(tree.children()), ["w"]);
    assert_eq!(values(tree.children().rev()), ["w"]);

    let node = tree.last().context("missing w")?;
    assert_eq!(node.value(), "w");
    assert_eq!(node.next().map(|n| n.value()), None);
    assert_eq!(node.last().map(|n| n.value()), Some("b"));
    assert_eq!(values(node.children().rev()), ["b", "a"]);
    assert_eq!(node.span(), &Span::new(0, 2));
    Ok(())
}

#[test]
fn close_at_root_after_sibling_updates_last() -> Result<()> {
    let mut tree = syntree::Builder::new();

    tree.token("x", 1)?;
    let c = tree.checkpoint()?;
    tree.token("a", 1)?;
    tree.token("b", 1)?;
    tree.close_at(&c, "w")?;

    let tree = tree.build()?;

    assert_eq!(values(tree.children()), ["x", "w"]);
    assert_eq!(values(tree.children().rev()), ["w", "x"]);

    let node = tree.last().context("missing w")?;
    assert_eq!(node.value(), "w");
    assert_eq!(node.prev().map(|n| n.value()), Some("x"));
    assert_eq!(node.next().map(|n| n.value()), None);
    assert_eq!(values(node.children().rev()), ["b", "a"]);
    Ok(())
}

#[test]
fn close_at_nested_updates_parent_last() -> Result<()> {
    let mut tree = syntree::Builder::new();

    tree.open("root")?;
    let c = tree.checkpoint()?;
    tree.token("a", 1)?;
    tree.token("b", 1)?;
    tree.close_at(&c, "w")?;
    tree.close()?;

    let tree = tree.build()?;

    let root = tree.first().context("missing root")?;
    assert_eq!(root.last().map(|n| n.value()), Some("w"));
    assert_eq!(values(root.children()), ["w"]);
    assert_eq!(values(root.children().rev()), ["w"]);

    let node = root.last().context("missing w")?;
    assert_eq!(node.parent().map(|n| n.value()), Some("root"));
    assert_eq!(node.next().map(|n| n.value()), None);
    assert_eq!(values(node.children().rev()), ["b", "a"]);
    Ok(())
}

#[test]
fn balanced_checkpoint_last() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let c = tree.checkpoint()?;
    tree.open(Number)?;
    tree.token(Lit, 2)?;
    tree.close()?;
    tree.token(Whitespace, 3)?;
    tree.open(Number)?;
    tree.token(Lit, 2)?;
    tree.close()?;
    tree.close_at(&c, Root)?;

    let tree = tree.build()?;

    assert_eq!(values(tree.children().rev()), [Root]);
    let root = tree.last().context("missing root")?;
    assert_eq!(values(root.children().rev()), [Number, Whitespace, Number]);
    Ok(())
}

#[test]
fn close_at_with_root_updates_last() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let c = tree.checkpoint()?;
    tree.token("a", 1)?;
    tree.token("b", 1)?;
    let w = tree.close_at_with(&c, "w", Span::new(0, 5))?;

    let tree = tree.build()?;

    assert_eq!(tree.last().map(|n| n.id()), Some(w));
    assert_eq!(values(tree.children().rev()), ["w"]);

    let node = tree.last().context("missing w")?;
    assert_eq!(node.span(), &Span::new(0, 5));
    assert_eq!(node.next().map(|n| n.value()), None);
    assert_eq!(values(node.children().rev()), ["b", "a"]);
    Ok(())
}

#[test]
fn close_at_with_nested_updates_parent_last() -> Result<()> {
    let mut tree = syntree::Builder::new();

    tree.open("root")?;
    tree.token("x", 1)?;
    let c = tree.checkpoint()?;
    tree.token("a", 1)?;
    tree.token("b", 1)?;
    tree.close_at_with(&c, "w", Span::new(1, 3))?;
    tree.close()?;

    let tree = tree.build()?;

    let root = tree.first().context("missing root")?;
    assert_eq!(root.last().map(|n| n.value()), Some("w"));
    assert_eq!(values(root.children()), ["x", "w"]);
    assert_eq!(values(root.children().rev()), ["w", "x"]);

    let node = root.last().context("missing w")?;
    assert_eq!(node.span(), &Span::new(1, 3));
    assert_eq!(node.next().map(|n| n.value()), None);
    assert_eq!(values(node.children().rev()), ["b", "a"]);
    Ok(())
}

#[test]
fn close_at_followed_by_tokens() -> Result<()> {
    let mut tree = syntree::Builder::new();

    tree.open("root")?;
    let c = tree.checkpoint()?;
    tree.token("a", 1)?;
    tree.token("b", 1)?;
    tree.close_at(&c, "w")?;
    tree.token("c", 1)?;
    tree.close()?;
    let c = tree.checkpoint()?;
    tree.token("d", 1)?;
    tree.close_at(&c, "v")?;
    tree.token("e", 1)?;

    let tree = tree.build()?;

    assert_eq!(values(tree.children()), ["root", "v", "e"]);
    assert_eq!(values(tree.children().rev()), ["e", "v", "root"]);

    let root = tree.first().context("missing root")?;
    assert_eq!(values(root.children()), ["w", "c"]);
    assert_eq!(values(root.children().rev()), ["c", "w"]);
    Ok(())
}

#[test]
fn close_at_error() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let c = tree.checkpoint()?;
    tree.open("child")?;
    tree.token("a", 1)?;
    assert_eq!(tree.close_at(&c, "w"), Err(Error::CloseAtError));
    assert_eq!(
        tree.close_at_with(&c, "w", Span::new(0, 1)),
        Err(Error::CloseAtError)
    );
    tree.close()?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        "child" => {
            ("a", 1)
        }
    };

    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn close_at_empty_checkpoint_reused() -> Result<()> {
    let mut tree = syntree::Builder::new();

    tree.token("x", 1)?;
    let c = tree.checkpoint()?;
    let inner = tree.close_at(&c, "inner")?;
    let outer = tree.close_at(&c, "outer")?;
    assert_ne!(inner, outer);

    let tree = tree.build()?;

    let expected = syntree::tree! {
        ("x", 1),
        "outer" => {
            "inner" => {}
        }
    };

    assert_eq!(tree, expected);
    assert_eq!(tree.last().map(|n| n.id()), Some(outer));
    assert_eq!(values(tree.children().rev()), ["outer", "x"]);

    let node = tree.last().context("missing outer")?;
    assert_eq!(node.first().map(|n| n.id()), Some(inner));
    assert_eq!(node.last().map(|n| n.id()), Some(inner));
    assert_eq!(node.span(), &Span::point(1));
    Ok(())
}

syntree::flavor! {
    struct FlavorU8 {
        type Index = u32;
        type Width = u8;
    }
}

#[test]
fn close_at_pointer_width_limit() -> Result<()> {
    let mut tree: syntree::Builder<u8, FlavorU8> = syntree::Builder::new_with();

    let c = tree.checkpoint()?;

    for d in 0..u8::MAX - 1 {
        tree.token(d, 1)?;
    }

    let w = tree.close_at(&c, 254)?;
    assert_eq!(w.get(), 254);

    assert_eq!(tree.checkpoint().err(), Some(Error::Overflow));
    assert_eq!(tree.close_at(&c, 255), Err(Error::Overflow));
    assert_eq!(tree.token(255, 1), Err(Error::Overflow));

    let tree = tree.build()?;

    assert_eq!(tree.len(), 255);
    assert_eq!(tree.last().map(|n| n.id()), Some(w));
    assert_eq!(values(tree.children().rev()), [254]);

    let node = tree.last().context("missing wrapper")?;
    assert_eq!(node.last().map(|n| n.value()), Some(253));
    assert_eq!(node.children().rev().count(), 254);
    assert_eq!(node.span(), &Span::new(0, 254));
    Ok(())
}
