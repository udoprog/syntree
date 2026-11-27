//! Building trees by repeatedly wrapping from one checkpoint must stay linear.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use syntree::{Builder, FlavorDefault, Tree};

type Built = Tree<&'static str, FlavorDefault>;

/// Build `a + a + ... + a` with `n` additions, wrapping left-associatively by
/// closing the same checkpoint after each operand.
fn left_assoc(n: usize) -> Result<Built> {
    let mut b = Builder::new();

    let c = b.checkpoint()?;
    b.token("a", 1)?;

    for _ in 0..n {
        b.token("+", 1)?;
        b.token("a", 1)?;
        b.close_at(c, "add")?;
    }

    Ok(b.build()?)
}

/// Build `a a ... a` with `n` tokens and a checkpoint before each, then close
/// the checkpoints innermost first so each wrapper nests the previous one.
fn nested(n: usize) -> Result<Built> {
    let mut b = Builder::new();
    let mut checkpoints = Vec::with_capacity(n);

    for _ in 0..n {
        checkpoints.push(b.checkpoint()?);
        b.token("a", 1)?;
    }

    for c in checkpoints.into_iter().rev() {
        b.close_at(c, "wrap")?;
    }

    Ok(b.build()?)
}

fn best_of(n: usize, f: fn(usize) -> Result<Built>) -> Result<Duration> {
    let mut best = Duration::MAX;

    for _ in 0..5 {
        let start = Instant::now();
        let tree = f(n)?;
        best = best.min(start.elapsed());
        drop(tree);
    }

    Ok(best)
}

fn assert_linear(f: fn(usize) -> Result<Built>) -> Result<()> {
    const N: usize = 5_000;

    let small = best_of(N, f)?;
    let large = best_of(N * 4, f)?;

    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);

    // Linear is about 4x, quadratic about 16x.
    assert!(
        ratio < 8.0,
        "4x the input took {ratio:.1}x as long ({small:?} -> {large:?})"
    );

    Ok(())
}

#[test]
fn left_assoc_close_at_is_linear() -> Result<()> {
    assert_linear(left_assoc)
}

#[test]
fn nested_close_at_is_linear() -> Result<()> {
    assert_linear(nested)
}

#[test]
fn left_assoc_close_at_keeps_token_parents() -> Result<()> {
    let tree = left_assoc(3)?;

    let expected = syntree::tree! {
        "add" => {
            "add" => {
                "add" => {
                    ("a", 1),
                    ("+", 1),
                    ("a", 1)
                },
                ("+", 1),
                ("a", 1)
            },
            ("+", 1),
            ("a", 1)
        }
    };

    assert_eq!(tree, expected);

    // Each token's parent is the innermost `add` covering it: the first
    // three belong to the deepest, then two to each enclosing one.
    let depths = [3, 3, 3, 2, 2, 1, 1];

    for (index, &depth) in depths.iter().enumerate() {
        let token = tree
            .node_with_range(index..index + 1)
            .with_context(|| format!("missing token at {index}"))?;
        let parent = token.parent().context("missing parent")?;
        assert_eq!(parent.value(), "add");
        assert_eq!(parent.ancestors().count(), depth, "token {index}");
    }

    Ok(())
}

#[test]
fn close_at_with_gaps_and_children_keeps_token_parents() -> Result<()> {
    let mut b = Builder::new();

    b.open("root")?;
    b.token("pre", 1)?;

    let c = b.checkpoint()?;
    b.token("x", 1)?;
    b.open("child")?;
    b.token("y", 1)?;
    b.open("inner")?;
    b.token("z", 1)?;
    b.close()?;
    b.close()?;
    b.set_cursor(5);
    b.token("w", 1)?;
    b.open("child")?;
    b.close()?;
    b.token("v", 1)?;
    b.close_at(c, "wrap")?;

    b.token("post", 1)?;
    b.close()?;

    let tree = b.build()?;

    let parent_of = |range: core::ops::Range<usize>| -> Result<&'static str> {
        let token = tree
            .node_with_range(range.clone())
            .with_context(|| format!("missing token at {range:?}"))?;
        Ok(token.parent().context("missing parent")?.value())
    };

    assert_eq!(parent_of(0..1)?, "root");
    assert_eq!(parent_of(1..2)?, "wrap");
    assert_eq!(parent_of(2..3)?, "child");
    assert_eq!(parent_of(3..4)?, "inner");
    assert_eq!(parent_of(5..6)?, "wrap");
    assert_eq!(parent_of(6..7)?, "wrap");
    assert_eq!(parent_of(7..8)?, "root");

    Ok(())
}
