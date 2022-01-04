use anyhow::Result;
use syntree::{print, Span, TreeBuilder};

fn main() -> Result<()> {
    let mut b = TreeBuilder::<u32>::new();

    let c = b.checkpoint();

    b.start_node(1);
    b.token(2, Span::new(1, 2));
    b.end_node()?;

    b.token(4, Span::new(2, 5));

    b.start_node(1);
    b.token(2, Span::new(5, 7));
    b.token(2, Span::new(7, 9));
    b.end_node()?;

    b.insert_node_at(c, 0);

    let tree = b.build()?;

    print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
