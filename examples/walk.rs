use anyhow::Result;
use syntree::print;

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

    for n in tree.walk() {
        dbg!(n.data());
    }

    print::print(&mut std::io::stdout(), &tree)?;
    Ok(())
}
