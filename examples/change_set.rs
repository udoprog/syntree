use anyhow::{Context, Result};
use syntree::{print, ChangeSet};

fn main() -> Result<()> {
    let tree = syntree::tree! {
        "root" => {
            "child" => {
                ("lit", 4),
                ("lit", 3),
            },
            ("whitespace", 4),
        }
    };

    let mut change_set = ChangeSet::new();

    let child = tree
        .first()
        .and_then(|n| n.first())
        .and_then(|n| n.first())
        .context("missing child")?;

    change_set.remove(child.id());

    let tree = change_set.modify(&tree)?;

    print::print(std::io::stdout(), &tree)?;
    Ok(())
}
