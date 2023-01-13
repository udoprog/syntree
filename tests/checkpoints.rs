use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    ROOT,
    NUMBER,
    LIT,
    WHITESPACE,
}

use Syntax::{LIT, NUMBER, ROOT, WHITESPACE};

#[test]
fn balanced_checkpoint() -> Result<()> {
    let mut tree = syntree::Builder::new();

    let c = tree.checkpoint()?;

    tree.open(NUMBER)?;
    tree.token(LIT, 2)?;
    tree.close()?;

    tree.token(WHITESPACE, 3)?;

    tree.open(NUMBER)?;
    tree.token(LIT, 2)?;
    tree.token(LIT, 2)?;
    tree.close()?;

    tree.close_at(&c, ROOT)?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        ROOT => {
            NUMBER => {
                (LIT, 2)
            },
            (WHITESPACE, 3),
            NUMBER => {
                (LIT, 2),
                (LIT, 2)
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
