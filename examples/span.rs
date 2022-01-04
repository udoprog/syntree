use anyhow::Result;
use syntree::TreeBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Number,
    Lit,
    Whitespace,
}

fn main() -> Result<()> {
    let tree = syntree::tree! {
        "root" => {
            "c1" => {
                "c2",
                "c3",
                "c4",
            },
            "c5",
            "c6"
        },
        "root2" => {
            "c7" => {
                "c8",
                "c9",
                "c10",
            },
            "c11",
            "c12" => {
                "c13",
                "c14",
                "c15",
            }
        }
    };

    let mut b = TreeBuilder::new();

    let c = b.checkpoint();

    b.start_node(Syntax::Number);
    b.token(Syntax::Lit, 2);
    b.end_node()?;

    b.token(Syntax::Whitespace, 3);

    b.start_node(Syntax::Number);
    b.token(Syntax::Lit, 2);
    b.token(Syntax::Lit, 2);
    b.end_node()?;

    b.insert_node_at(c, Syntax::Root);

    let tree = b.build()?;

    for node in tree.walk() {
        dbg!(node.data());
    }

    syntree::print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
