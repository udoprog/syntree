use anyhow::Result;
use syntree::{print, TreeBuilder};

fn main() -> Result<()> {
    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint();
    tree.token("lonely", 1);
    tree.close_at(c, "root")?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint();

    tree.open("child1");
    tree.token("token1", 1);
    tree.token("token2", 1);
    tree.close()?;

    tree.token("whitespace", 3);

    tree.open("child2");
    tree.token("token3", 3);
    tree.token("token4", 3);
    tree.close()?;

    tree.close_at(c, "root")?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    Ok(())
}
