use anyhow::Result;
use syntree::{print, Span, TreeBuilder};

#[derive(Debug, Clone, Copy)]
enum Syntax {
    Root,
    Number,
    Lit,
    Whitespace,
}

fn main() -> Result<()> {
    let mut b = TreeBuilder::new();

    let c = b.checkpoint();

    b.start_node(Syntax::Number);
    b.token(Syntax::Lit, Span::new(1, 2));
    b.end_node()?;

    b.token(Syntax::Whitespace, Span::new(2, 5));

    b.start_node(Syntax::Number);
    b.token(Syntax::Lit, Span::new(5, 7));
    b.token(Syntax::Lit, Span::new(7, 9));
    b.end_node()?;

    b.insert_node_at(c, Syntax::Root);

    let tree = b.build()?;

    print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
