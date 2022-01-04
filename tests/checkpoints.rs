use syntree::{Span, TreeBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Number,
    Lit,
    Whitespace,
}

#[test]
fn balanced_checkpoint() -> anyhow::Result<()> {
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

    let expected = syntree::tree! {
        >> Syntax::Root,
            >> Syntax::Number,
                + Syntax::Lit, (1, 2)
            <<
            + Syntax::Whitespace, (2, 5)
            >> Syntax::Number,
                + Syntax::Lit, (5, 7)
                + Syntax::Lit, (7, 9)
            <<
        <<
    };

    assert_eq!(expected, tree);
    Ok(())
}
