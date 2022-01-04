use anyhow::Result;
use syntree::{print, TreeBuilder};

fn main() -> Result<()> {
    let mut b = TreeBuilder::<u32>::new();

    let c = b.checkpoint();

    b.start_node(1);
    b.token(2, 1);
    b.token(3, 1);
    b.end_node()?;

    b.token(4, 3);

    b.start_node(5);
    b.token(6, 3);
    b.token(7, 3);
    b.end_node()?;

    b.insert_node_at(c, 0);

    let tree = b.build()?;

    print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
