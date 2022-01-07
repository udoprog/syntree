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
    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint()?;

    tree.open(Syntax::Number)?;
    tree.token(Syntax::Lit, 2)?;
    tree.close()?;

    tree.token(Syntax::Whitespace, 3)?;

    tree.open(Syntax::Number)?;
    tree.token(Syntax::Lit, 2)?;
    tree.token(Syntax::Lit, 2)?;
    tree.close()?;

    tree.close_at(c, Syntax::Root)?;

    let tree = tree.build()?;

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
