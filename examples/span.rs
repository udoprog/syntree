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

    if let Some(n) = tree.last() {
        for n in n.walk() {
            dbg!(n.data());
        }
    }

    syntree::print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
