use syntree::TreeBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    ROOT,
    NUMBER,
    LIT,
    WHITESPACE,
}

use Syntax::*;

#[test]
fn balanced_checkpoint() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = TreeBuilder::new();

    let c = tree.checkpoint()?;

    tree.open(NUMBER)?;
    tree.token(LIT, 2)?;
    tree.close()?;

    tree.token(WHITESPACE, 3)?;

    tree.open(NUMBER)?;
    tree.token(LIT, 2)?;
    tree.token(LIT, 2)?;
    tree.close()?;

    tree.close_at(c, ROOT)?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        ROOT => {
            NUMBER => {
                (LIT, 2)
            },
            (WHITESPACE, 3),
            NUMBER => {
                (LIT, 2),
                (LIT, 2)
            }
        }
    };

    assert_eq!(expected, tree);
    Ok(())
}
