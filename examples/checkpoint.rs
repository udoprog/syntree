use anyhow::Result;
use syntree::{print, TreeBuilder};

fn main() -> Result<()> {
    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint()?;
    tree.open("child")?;
    tree.token("lit", 3)?;
    tree.close()?;
    tree.close_at(c, "root")?;
    tree.token("sibling", 3)?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;
    Ok(())
}
