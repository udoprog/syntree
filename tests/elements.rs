//! Iteration which starts at a token, and the `Node`-like methods of
//! `Element`.

use syntree::{Builder, Span};

type Res = Result<(), Box<dyn core::error::Error>>;

fn values<'a, I>(it: I) -> Vec<&'static str>
where
    I: IntoIterator<Item = syntree::Element<'a, &'static str, syntree::FlavorDefault>>,
{
    it.into_iter().map(|e| e.value()).collect()
}

#[test]
fn token_starts() -> Res {
    let tree = syntree::tree! {
        "root" => {
            ("t1", 1),
            "a" => { ("x", 1) },
            ("t2", 1),
            "b" => {},
            ("t3", 1)
        },
        ("t4", 1)
    };

    let t2 = tree
        .walk()
        .find(|e| e.value() == "t2")
        .and_then(|e| e.into_token())
        .ok_or("missing t2")?;

    assert_eq!(values(t2.siblings()), ["t2", "b", "t3"]);
    assert_eq!(values(t2.walk()), ["t2", "b", "t3", "t4"]);
    assert_eq!(values(t2.walk().inside()), ["t2"]);
    assert_eq!(values(t2.walk_from()), ["b", "t3", "t4"]);
    assert_eq!(values(t2.ancestors()), ["t2", "root"]);
    assert_eq!(
        t2.walk()
            .with_depths()
            .map(|(d, e)| (d, e.value()))
            .collect::<Vec<_>>(),
        [(0, "t2"), (0, "b"), (0, "t3"), (-1, "t4")]
    );

    let t1 = tree.first().and_then(|e| e.first()).ok_or("missing t1")?;
    assert_eq!(values(t1.siblings()), ["t1", "a", "t2", "b", "t3"]);
    assert_eq!(
        t1.siblings()
            .skip_tokens()
            .map(|n| n.value())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );

    let t4 = tree.last().ok_or("missing t4")?;
    assert_eq!(values(t4.siblings()), ["t4"]);
    assert_eq!(values(t4.walk_from()), [] as [&str; 0]);
    assert_eq!(values(t4.ancestors()), ["t4"]);
    Ok(())
}

#[test]
fn token_starts_with_gaps() -> Res {
    let mut b = Builder::new();
    b.open("root")?;
    b.token_with("t1", Span::new(2, 3))?;
    b.open("a")?;
    b.token_with("x", Span::new(5, 6))?;
    b.close()?;
    b.token_with("t2", Span::new(8, 9))?;
    b.close()?;
    let tree = b.build()?;

    let t1 = tree.first().and_then(|e| e.first()).ok_or("missing t1")?;
    assert_eq!(values(t1.siblings()), ["t1", "a", "t2"]);
    assert_eq!(values(t1.walk()), ["t1", "a", "x", "t2"]);

    let x = tree.walk().find(|e| e.value() == "x").ok_or("missing x")?;
    assert_eq!(values(x.walk_from()), ["t2"]);
    assert_eq!(values(x.ancestors()), ["x", "a", "root"]);

    let found = x.find_preceding(|_| true).ok_or("missing preceding")?;
    assert_eq!(found.value(), "t1");
    Ok(())
}

#[test]
fn element_node_like() -> Res {
    let tree = syntree::tree! {
        "root" => {
            ("lit", 3),
            "child" => { ("ws", 1) }
        }
    };

    let root = tree.first().ok_or("missing root")?;
    let lit = root.first().ok_or("missing lit")?;
    let child = root.last().ok_or("missing child")?;

    assert!(lit.is_token());
    assert!(lit.first().is_none());
    assert!(lit.last().is_none());
    assert_eq!(lit.children().count(), 0);
    assert_eq!(lit.children().rev().count(), 0);
    assert!(!lit.has_children());

    assert_eq!(values(child.children()), ["ws"]);
    assert_eq!(child.first().map(|e| e.value()), Some("ws"));

    assert_eq!(lit.replace("other"), "lit");
    assert_eq!(lit.value(), "other");

    assert_eq!(
        format!("{lit:?}"),
        "Token { data: \"other\", span: (0, 3) }"
    );
    assert!(format!("{child:?}").starts_with("Node { data: \"child\""));
    Ok(())
}
