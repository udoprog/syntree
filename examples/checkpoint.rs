use anyhow::Result;
use syntree::{print, TreeBuilder};

fn main() -> Result<()> {
    let mut b = TreeBuilder::<u32>::new();

    let c = b.checkpoint();

    b.open(1);
    b.token(2, 1);
    b.token(3, 1);
    b.close()?;

    b.token(4, 3);

    b.open(5);
    b.token(6, 3);
    b.token(7, 3);
    b.close()?;

    b.close_at(c, 0);

    let tree = b.build()?;

    print::print(std::io::stdout(), &tree)?;
    Ok(())
}
