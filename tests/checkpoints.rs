use syntree::TreeBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Number,
    Lit,
    Whitespace,
}

#[test]
fn balanced_checkpoint() -> Result<(), Box<dyn std::error::Error>> {
    let mut b = TreeBuilder::new();

    let c = b.checkpoint();

    b.open(Syntax::Number);
    b.token(Syntax::Lit, 2);
    b.close()?;

    b.token(Syntax::Whitespace, 3);

    b.open(Syntax::Number);
    b.token(Syntax::Lit, 2);
    b.token(Syntax::Lit, 2);
    b.close()?;

    b.close_at(c, Syntax::Root);

    let tree = b.build()?;

    let expected = syntree::tree! {
        Syntax::Root => {
            Syntax::Number => {
                (Syntax::Lit, 2)
            },
            (Syntax::Whitespace, 3),
            Syntax::Number => {
                (Syntax::Lit, 2),
                (Syntax::Lit, 2)
            }
        }
    };

    assert_eq!(expected, tree);
    Ok(())
}
