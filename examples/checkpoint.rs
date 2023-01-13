use anyhow::{Context, Result};
use syntree::{print, TreeBuilder};

fn main() -> Result<()> {
    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint()?;
    tree.open("child")?;
    tree.token("lit", 3)?;
    tree.close()?;
    tree.close_at(&c, "root")?;
    tree.token("sibling", 3)?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    let child = tree.node_with_range(0..3).context("missing at 0..3")?;
    assert_eq!(*child.value(), "child");

    let lit = tree
        .first()
        .and_then(|n| n.first())
        .and_then(|n| n.first())
        .context("expected lit")?;
    assert_eq!(*lit.value(), "lit");

    let root = tree.first().context("missing root")?;
    assert_eq!(root.parent(), None);

    let root = lit.ancestors().last().context("missing root")?;
    assert_eq!(*root.value(), "root");
    Ok(())
}
