use anyhow::Result;

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

    for node in tree.walk() {
        dbg!(node.data());
    }

    Ok(())
}
