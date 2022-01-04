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
    let tree = syntree::tree! {
        Syntax::Root => {
            Syntax::Operation => {
                Syntax::Operation => {
                    (Syntax::Number, 4),
                    (Syntax::Plus, 1),
                    (Syntax::Number, 5)
                },
                (Syntax::Plus, 1),
                (Syntax::Number, 5)
            }
        }
    };

    let mut it = tree.walk();
    let mut has = true;

    while has {
        if let Some(n) = it.next() {
            dbg!(n.data());
        } else {
            has = false;
        }

        if let Some(n) = it.next_back() {
            dbg!(n.data());
        } else {
            has = false;
        }
    }

    print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
