use anyhow::{Context, Result};
use syntree::edit::ChangeSet;

use syntree::node::NodeOrToken;
use syntree::{Element, Flavor, FlavorDefault, Tree};

type T = Tree<&'static str, FlavorDefault>;
type P = <FlavorDefault as Flavor>::Pointer;
type E<'a> = Element<'a, &'static str, FlavorDefault>;

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

fn find<'a>(tree: &'a T, value: &str) -> Result<E<'a>> {
    tree.walk()
        .find(|n| n.value() == value)
        .with_context(|| format!("missing {value}"))
}

/// Node and token ids are separate index spaces, so an element is identified
/// by its kind and its id.
fn key(e: E<'_>) -> NodeOrToken<P, P> {
    match e {
        NodeOrToken::Node(n) => NodeOrToken::Node(n.id()),
        NodeOrToken::Token(t) => NodeOrToken::Token(t.id()),
    }
}

fn modify(tree: &T, remove: &[&str]) -> Result<T> {
    let mut change_set = ChangeSet::new();

    for value in remove {
        change_set.remove(find(tree, value)?);
    }

    Ok(change_set.modify(tree)?)
}

fn check_siblings<'a>(
    mut children: impl Iterator<Item = E<'a>>,
    parent: Option<E<'a>>,
    last: Option<E<'a>>,
) {
    let mut prev = None::<E<'a>>;

    for child in children.by_ref() {
        assert_eq!(
            child.parent().map(|n| n.id()),
            parent.and_then(|n| n.into_node()).map(|n| n.id()),
            "parent of {child:?}"
        );
        assert_eq!(child.prev().map(key), prev.map(key), "prev of {child:?}");
        check_siblings(child.children().with_trivia(), Some(child), child.last());
        prev = Some(child);
    }

    assert_eq!(last.map(key), prev.map(key), "last child of {parent:?}");
}

/// Assert that `actual` equals `expected`, including the links and span
/// indexes which `PartialEq` does not compare.
fn check(actual: &T, expected: &T) {
    assert_eq!(actual, expected);
    assert_eq!(actual.span(), expected.span());
    check_siblings(actual.children().with_trivia(), None, actual.last());

    for node in expected.walk().filter(|n| !n.has_children()) {
        let span = node.span();
        let a = actual.node_with_span(span).map(|n| (n.value(), n.span()));
        let e = expected.node_with_span(span).map(|n| (n.value(), n.span()));
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
            // The emptied node stays an interior node rather than becoming a
            // token.
            "child" => {},
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

#[test]
fn remove_by_id_and_gaps_compact() -> Result<()> {
    let mut b = syntree::Builder::new();
    b.open("root")?;
    let a = b.token_with("a", syntree::Span::new(0, 1))?;
    // Leaves a gap of 2 before `b`.
    b.token_with("b", syntree::Span::new(3, 5))?;
    let inner = b.open("inner")?;
    b.token("c", 1)?;
    b.close()?;
    b.close()?;
    let tree: T = b.build()?;

    let mut change_set = ChangeSet::new();
    change_set.remove_token(a);
    change_set.remove_node(inner);

    let expected = syntree::tree! {
        "root" => {
            ("b", 2),
        }
    };

    check(&change_set.modify(&tree)?, &expected);

    // Node and token ids are separate: removing node id `a` (the root, id 0)
    // is not removing token `a`.
    let mut change_set = ChangeSet::new();
    change_set.remove_node(a);
    assert!(change_set.modify(&tree)?.is_empty());
    Ok(())
}

/// `ROOT { ws CHILD { a ws b } c ws }` with every `ws` as trivia.
fn trivia_sample(remove_child: bool, remove_c: bool) -> Result<T> {
    let mut b = syntree::Builder::new();
    b.open("root")?;
    b.trivia("ws", 1)?;

    if !remove_child {
        b.open("child")?;
        b.token("a", 1)?;
        b.trivia("ws", 1)?;
        b.token("b", 2)?;
        b.close()?;
    }

    if !remove_c {
        b.token("c", 3)?;
    }

    b.trivia("ws", 1)?;
    b.close()?;
    Ok(b.build()?)
}

fn trivia_flags(tree: &T) -> Vec<(&'static str, bool)> {
    tree.walk()
        .with_trivia()
        .filter_map(|e| e.into_token())
        .map(|t| (t.value(), t.is_trivia()))
        .collect()
}

#[test]
fn modify_keeps_trivia() -> Result<()> {
    let tree = trivia_sample(false, false)?;

    let empty = ChangeSet::new().modify(&tree)?;
    check(&empty, &tree);
    assert_eq!(trivia_flags(&empty), trivia_flags(&tree));

    let actual = modify(&tree, &["c"])?;
    let expected = trivia_sample(false, true)?;
    check(&actual, &expected);
    assert_eq!(
        trivia_flags(&actual),
        [
            ("ws", true),
            ("a", false),
            ("ws", true),
            ("b", false),
            ("ws", true)
        ]
    );

    let actual = modify(&tree, &["child"])?;
    check(&actual, &trivia_sample(true, false)?);
    Ok(())
}

#[test]
fn modify_removes_trivia() -> Result<()> {
    let tree = trivia_sample(false, false)?;

    let ws = tree
        .walk()
        .with_trivia()
        .filter(|e| e.is_trivia())
        .last()
        .context("missing trivia")?;

    let mut change_set = ChangeSet::new();
    change_set.remove(ws);
    let actual = change_set.modify(&tree)?;

    assert_eq!(
        trivia_flags(&actual),
        [
            ("ws", true),
            ("a", false),
            ("ws", true),
            ("b", false),
            ("c", false)
        ]
    );
    Ok(())
}
