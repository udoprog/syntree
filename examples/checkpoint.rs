use anyhow::Result;
use syntree::{print, TreeBuilder};

fn main() -> Result<()> {
    let mut tree = TreeBuilder::<u32>::new();

    let c = tree.checkpoint();
    tree.token(2, 1);
    tree.close_at(c, 0)?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    let mut tree = TreeBuilder::<u32>::new();

    let c = tree.checkpoint();

    tree.open(1);
    tree.token(2, 1);
    tree.token(3, 1);
    tree.close()?;

    tree.token(4, 3);

    tree.open(5);
    tree.token(6, 3);
    tree.token(7, 3);
    tree.close()?;

    tree.close_at(c, 0)?;

    let tree = tree.build()?;

    print::print(std::io::stdout(), &tree)?;

    Ok(())
}
