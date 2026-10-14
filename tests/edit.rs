use anyhow::{Context, Result};
use syntree::edit::ChangeSet;

use syntree::{FlavorDefault, Node, Tree};

type T = Tree<&'static str, FlavorDefault>;

fn sample() -> Result<T> {
    Ok(syntree::tree! {
        "root" => {
            "child" => {
                ("a", 1),
                ("b", 2),
            },
            ("ws", 3),
            ("c", 4),
        }
    })
}

fn find<'a>(tree: &'a T, value: &str) -> Result<Node<'a, &'static str, FlavorDefault>> {
    tree.walk()
        .find(|n| n.value() == value)
        .with_context(|| format!("missing {value}"))
}

fn modify(tree: &T, remove: &[&str]) -> Result<T> {
    let mut change_set = ChangeSet::new();

    for value in remove {
        change_set.remove(find(tree, value)?.id());
    }

    Ok(change_set.modify(tree)?)
}

fn check_siblings<'a>(
    mut children: impl Iterator<Item = Node<'a, &'static str, FlavorDefault>>,
    parent: Option<Node<'a, &'static str, FlavorDefault>>,
    last: Option<Node<'a, &'static str, FlavorDefault>>,
) {
    let mut prev = None::<Node<'a, &'static str, FlavorDefault>>;

    for child in children.by_ref() {
        assert_eq!(
            child.parent().map(|n| n.id()),
            parent.map(|n| n.id()),
            "parent of {child:?}"
        );
        assert_eq!(
            child.prev().map(|n| n.id()),
            prev.map(|n| n.id()),
            "prev of {child:?}"
        );
        check_siblings(child.children(), Some(child), child.last());
        prev = Some(child);
    }

    assert_eq!(
        last.map(|n| n.id()),
        prev.map(|n| n.id()),
        "last child of {parent:?}"
    );
}

/// Assert that `actual` equals `expected`, including the links and span
/// indexes which `PartialEq` does not compare.
fn check(actual: &T, expected: &T) {
    assert_eq!(actual, expected);
    assert_eq!(actual.span(), expected.span());
    check_siblings(actual.children(), None, actual.last());

    for node in expected.walk().filter(|n| !n.has_children()) {
        let span = *node.span();
        let a = actual.node_with_span(span).map(|n| (n.value(), *n.span()));
        let e = expected
            .node_with_span(span)
            .map(|n| (n.value(), *n.span()));
        assert_eq!(a, e, "node_with_span({span:?})");
    }
}

#[test]
fn empty_change_set_round_trip() -> Result<()> {
    let trees = [
        sample()?,
        syntree::tree! {
            "r" => {
                "m" => {
                    ("x", 2)
                }
            }
        },
        syntree::tree! {
            "r" => {
                "m" => {
                    "n" => {
                        ("x", 1),
                        ("y", 2),
                    },
                    ("z", 3),
                },
                ("w", 4),
            },
            "s" => {
                ("v", 5),
            }
        },
    ];

    for tree in &trees {
        check(&ChangeSet::new().modify(tree)?, tree);
    }

    let tree = &trees[1];
    let out = ChangeSet::new().modify(tree)?;
    let r = out.first().context("missing r")?;
    assert_eq!(r.range(), 0..2);
    Ok(())
}

#[test]
fn remove_last_child() -> Result<()> {
    let tree = sample()?;

    let expected = syntree::tree! {
        "root" => {
            "child" => {
                ("a", 1),
            },
            ("ws", 3),
            ("c", 4),
        }
    };

    check(&modify(&tree, &["b"])?, &expected);
    Ok(())
}

#[test]
fn remove_first_child() -> Result<()> {
    let tree = sample()?;

    let expected = syntree::tree! {
        "root" => {
            "child" => {
                ("b", 2),
            },
            ("ws", 3),
            ("c", 4),
        }
    };

    check(&modify(&tree, &["a"])?, &expected);
    Ok(())
}

#[test]
fn remove_all_children() -> Result<()> {
    let tree = sample()?;

    let expected = syntree::tree! {
        "root" => {
            "child",
            ("ws", 3),
            ("c", 4),
        }
    };

    check(&modify(&tree, &["a", "b"])?, &expected);
    Ok(())
}

#[test]
fn remove_two_adjacent() -> Result<()> {
    let tree = sample()?;

    let expected = syntree::tree! {
        "root" => {
            ("c", 4),
        }
    };

    check(&modify(&tree, &["child", "ws"])?, &expected);

    let expected = syntree::tree! {
        "root" => {
            "child" => {
                ("a", 1),
                ("b", 2),
            },
        }
    };

    check(&modify(&tree, &["ws", "c"])?, &expected);
    Ok(())
}

#[test]
fn remove_two_non_adjacent() -> Result<()> {
    let tree = sample()?;

    let expected = syntree::tree! {
        "root" => {
            ("ws", 3),
        }
    };

    check(&modify(&tree, &["child", "c"])?, &expected);

    let expected = syntree::tree! {
        "root" => {
            "child" => {
                ("b", 2),
            },
            ("ws", 3),
        }
    };

    check(&modify(&tree, &["a", "c"])?, &expected);
    Ok(())
}

#[test]
fn remove_inside_removed() -> Result<()> {
    let tree = sample()?;

    let expected = syntree::tree! {
        "root" => {
            ("ws", 3),
            ("c", 4),
        }
    };

    check(&modify(&tree, &["child", "b"])?, &expected);
    Ok(())
}

#[test]
fn remove_nested_subtree() -> Result<()> {
    let tree = syntree::tree! {
        "r" => {
            "m" => {
                "n" => {
                    ("x", 1),
                    ("y", 2),
                },
                ("z", 3),
            },
            ("w", 4),
        }
    };

    let expected = syntree::tree! {
        "r" => {
            "m" => {
                ("z", 3),
            },
            ("w", 4),
        }
    };

    check(&modify(&tree, &["n"])?, &expected);

    let expected = syntree::tree! {
        "r" => {
            "m" => {
                "n" => {
                    ("x", 1),
                    ("y", 2),
                },
            },
            ("w", 4),
        }
    };

    check(&modify(&tree, &["z"])?, &expected);
    Ok(())
}

#[test]
fn remove_root_first_and_last() -> Result<()> {
    let tree = syntree::tree! {
        ("a", 1),
        "b" => {
            ("c", 2),
        },
        ("d", 3),
    };

    let expected = syntree::tree! {
        "b" => {
            ("c", 2),
        },
        ("d", 3),
    };

    check(&modify(&tree, &["a"])?, &expected);

    let expected = syntree::tree! {
        ("a", 1),
        "b" => {
            ("c", 2),
        },
    };

    check(&modify(&tree, &["d"])?, &expected);

    let expected = syntree::tree! {
        "b" => {
            ("c", 2),
        },
    };

    check(&modify(&tree, &["a", "d"])?, &expected);
    Ok(())
}

#[test]
fn remove_everything() -> Result<()> {
    let tree = sample()?;
    let out = modify(&tree, &["root"])?;
    assert!(out.is_empty());
    assert!(out.first().is_none());
    assert!(out.last().is_none());
    assert_eq!(out.span().len(), 0);
    Ok(())
}
