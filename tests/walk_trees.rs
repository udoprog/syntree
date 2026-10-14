#[test]
fn walk_trees1() -> Result<(), Box<dyn core::error::Error>> {
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
fn walk_trees2() -> Result<(), Box<dyn core::error::Error>> {
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

#[test]
fn tree_walk_inside_covers_all_roots() -> Result<(), Box<dyn core::error::Error>> {
    let tree = syntree::tree! {
        "root" => {
            "a",
            "b" => {
                "b1"
            },
            "c",
            "d",
        },
        "root2" => {
            "e"
        }
    };

    let all = tree.walk().map(|n| n.value()).collect::<Vec<_>>();
    let inside = tree.walk().inside().map(|n| n.value()).collect::<Vec<_>>();

    assert_eq!(all, ["root", "a", "b", "b1", "c", "d", "root2", "e"]);
    assert_eq!(inside, all);
    Ok(())
}

#[test]
fn inside_after_partial_advance() -> Result<(), Box<dyn core::error::Error>> {
    let tree = syntree::tree! {
        "root" => {
            "a",
            "b" => {
                "b1"
            },
            "c",
            "d",
        },
        "root2" => {
            "e"
        }
    };

    let root = tree.first().ok_or("missing root")?;

    // A node walk stays scoped to the node it started at.
    let mut walk = root.walk();
    assert_eq!(walk.next().map(|n| n.value()), Some("root"));
    assert_eq!(walk.next().map(|n| n.value()), Some("a"));
    let values = walk.inside().map(|n| n.value()).collect::<Vec<_>>();
    assert_eq!(values, ["b", "b1", "c", "d"]);

    // A tree walk stays scoped to the whole tree.
    let mut walk = tree.walk();
    assert_eq!(walk.next().map(|n| n.value()), Some("root"));
    assert_eq!(walk.next().map(|n| n.value()), Some("a"));
    let values = walk.inside().map(|n| n.value()).collect::<Vec<_>>();
    assert_eq!(values, ["b", "b1", "c", "d", "root2", "e"]);
    Ok(())
}
