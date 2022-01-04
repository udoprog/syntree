use anyhow::Result;
use syntree::{print, TreeBuilder};

#[derive(Debug, Clone, Copy)]
enum Syntax {
    Root,
    Operation,
    Number,
    Plus,
}

fn main() -> Result<()> {
    let mut b = TreeBuilder::new();

    b.start_node(Syntax::Root);
    b.end_node()?;

    b.start_node(Syntax::Root);
    b.start_node(Syntax::Operation);

    b.start_node(Syntax::Number);
    b.token(Syntax::Number, 4);
    b.end_node()?;

    b.start_node(Syntax::Plus);
    b.token(Syntax::Plus, 1);
    b.end_node()?;

    b.start_node(Syntax::Number);
    b.token(Syntax::Number, 5);
    b.end_node()?;

    b.end_node()?;
    b.end_node()?;

    let tree = b.build()?;

    print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
