use anyhow::Result;
use syntree::{Builder, FlavorDefault, Tree};

syntree::flavor! {
    struct FlavorU8 {
        type Index = u32;
        type Width = u8;
    }
}

fn build<F>() -> Result<Builder<&'static str, F>>
where
    F: syntree::Flavor<Error = core::convert::Infallible, Length = usize>,
{
    let mut b = Builder::new_with();
    b.open("root")?;
    b.token("lit", 3)?;
    b.close()?;
    Ok(b)
}

#[test]
fn clone_tree_default_flavor() -> Result<()> {
    let tree: Tree<&str, FlavorDefault> = syntree::tree! { "a" => { ("b", 1) } };
    let cloned = tree.clone();
    assert_eq!(tree, cloned);
    assert_eq!(cloned.node_with_range(0..1).map(|n| n.value()), Some("b"));
    Ok(())
}

#[test]
fn clone_tree_custom_flavor() -> Result<()> {
    let tree = build::<FlavorU8>()?.build()?;
    let cloned = tree.clone();
    assert_eq!(tree, cloned);
    assert_eq!(cloned.node_with_range(0..3).map(|n| n.value()), Some("lit"));
    Ok(())
}

#[test]
fn clone_builder() -> Result<()> {
    let a = build::<FlavorDefault>()?;
    assert_eq!(a.clone().build()?, a.build()?);

    let a = build::<FlavorU8>()?;
    assert_eq!(a.clone().build()?, a.build()?);
    Ok(())
}

#[test]
fn debug_builder() -> Result<()> {
    let b: Builder<&str> = Builder::new();
    assert!(format!("{b:?}").starts_with("Builder"));

    let b = build::<FlavorU8>()?;
    let s = format!("{b:?}");
    assert!(s.contains("\"lit\""), "{s}");
    Ok(())
}

#[test]
fn debug_flavor() {
    assert_eq!(format!("{FlavorU8:?}"), "FlavorU8");
}

/// A builder cloned for speculative parsing hands out its own checkpoints, so
/// closing one on the clone leaves the original's checkpoint untouched.
#[test]
fn checkpoint_on_clone_does_not_affect_original() -> Result<()> {
    let mut a = Builder::new();
    let c = a.checkpoint()?;

    let mut b = a.clone();
    let bc = b.checkpoint()?;
    b.token("lit", 1)?;
    b.close_at(&bc, "speculative")?;

    a.token("lit", 1)?;
    a.close_at(&c, "root")?;

    assert_eq!(a.build()?, syntree::tree! { "root" => { ("lit", 1) } });
    assert_eq!(
        b.build()?,
        syntree::tree! { "speculative" => { ("lit", 1) } }
    );
    Ok(())
}
