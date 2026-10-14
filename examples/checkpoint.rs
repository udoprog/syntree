use anyhow::{Context, Result};
use syntree::{print, Builder};

fn main() -> Result<()> {
    let mut tree = Builder::new();

    let c = tree.checkpoint()?;
    tree.open("child")?;
    tree.token("lit", 3)?;
    tree.close()?;
    tree.close_at(&c, "root")?;
    tree.token("sibling", 3)?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    let lit = tree.node_with_range(0..3).context("missing at 0..3")?;
    assert_eq!(lit.value(), "lit");

    let child = lit.parent().context("missing child")?;
    assert_eq!(child.value(), "child");

    let root = tree.first().context("missing root")?;
    assert_eq!(root.parent(), None);

    let root = lit.ancestors().last().context("missing root")?;
    assert_eq!(root.value(), "root");
    Ok(())
}
