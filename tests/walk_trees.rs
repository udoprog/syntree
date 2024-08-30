#[test]
fn walk_trees1() -> Result<(), Box<dyn std::error::Error>> {
    let tree = syntree::tree! {
        "n1" => {
            "n2",
            "n3" => {
                "n4"
            },
            "n5",
        },
        "n6" => {
            "n7"
        }
    };

    let n1 = tree.first().ok_or("missing n1")?;

    let values = n1
        .children()
        .flat_map(|n| n.walk().inside())
        .map(|n| n.value())
        .collect::<Vec<_>>();

    assert_eq!(values, ["n2", "n3", "n4", "n5"]);
    Ok(())
}

#[test]
fn walk_trees2() -> Result<(), Box<dyn std::error::Error>> {
    let tree = syntree::tree! {
        "n1" => {
            "n2",
            "n3",
            "n5",
        },
        "n6" => {
            "n7"
        }
    };

    let n1 = tree.first().ok_or("missing n1")?;

    let values = n1
        .children()
        .flat_map(|n| n.walk().inside())
        .map(|n| n.value())
        .collect::<Vec<_>>();

    assert_eq!(values, ["n2", "n3", "n5"]);
    Ok(())
}
