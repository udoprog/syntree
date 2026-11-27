//! Pre-sizing a builder.

use syntree::{Builder, Empty, Error, FlavorDefault};

#[test]
fn presized_builder_does_not_grow() -> Result<(), Error> {
    const TOKENS: usize = 1000;

    let mut tree = Builder::<_, FlavorDefault>::with_capacities(1, TOKENS)?;

    tree.open("root")?;

    for _ in 0..TOKENS {
        tree.token("token", 1)?;
    }

    tree.close()?;

    let tree = tree.build()?;
    assert_eq!(tree.len(), TOKENS + 1);
    // Exact allocation means storage never grew past the hint.
    assert_eq!(tree.capacity(), TOKENS + 1);

    let node = tree
        .node_with_range(500..501)
        .ok_or(Error::MissingNode(0))?;
    assert_eq!(node.span().range(), 500..501);
    Ok(())
}

#[test]
fn reserve_grows_once() -> Result<(), Error> {
    let mut tree = Builder::new();
    tree.reserve(0, 100)?;

    for _ in 0..100 {
        tree.token("token", 1)?;
    }

    let tree = tree.build()?;
    assert_eq!(tree.capacity(), 100);
    Ok(())
}

#[test]
fn oversized_hint_is_not_an_error() -> Result<(), Error> {
    let mut tree = Builder::<_, FlavorDefault>::with_capacities(usize::MAX, usize::MAX)?;
    tree.reserve(usize::MAX, usize::MAX)?;
    tree.token("token", 1)?;
    assert_eq!(tree.build()?.len(), 1);
    Ok(())
}

syntree::flavor! {
    struct FlavorEmpty {
        type Index = Empty;
    }
}

#[test]
fn empty_index_oversized_hint() -> Result<(), Error> {
    let mut tree = Builder::<_, FlavorEmpty>::with_capacities(1, usize::MAX)?;
    tree.token("token", Empty)?;
    assert_eq!(tree.build()?.len(), 1);
    Ok(())
}
