use anyhow::Result;

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
