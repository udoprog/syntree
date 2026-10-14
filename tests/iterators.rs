use syntree::node::{
    Ancestors, Children, Inside, Siblings, SkipTokens, Walk, WalkEvents, WithDepths,
};
use syntree::FlavorDefault;

type S = &'static str;

#[derive(Debug, Default)]
#[allow(dead_code)]
struct Holder<'a> {
    ancestors: Ancestors<'a, S, FlavorDefault>,
    children: Children<'a, S, FlavorDefault>,
    siblings: Siblings<'a, S, FlavorDefault>,
    skip_tokens: SkipTokens<Children<'a, S, FlavorDefault>>,
    walk: Walk<'a, S, FlavorDefault>,
    with_depths: WithDepths<'a, S, FlavorDefault>,
    inside: Inside<'a, S, FlavorDefault>,
    walk_events: WalkEvents<'a, S, FlavorDefault>,
}

#[test]
fn debug_iterators() -> Result<(), Box<dyn core::error::Error>> {
    let tree = syntree::tree! {
        "root" => {
            "c1" => {
                ("t1", 1)
            },
            "c2" => {}
        }
    };

    let root = tree.first().ok_or("missing root")?;
    let c1 = root.first().ok_or("missing c1")?;

    let holder = Holder {
        ancestors: c1.ancestors(),
        children: root.children(),
        siblings: c1.siblings(),
        skip_tokens: root.children().skip_tokens(),
        walk: root.walk(),
        with_depths: root.walk().with_depths(),
        inside: root.walk().inside(),
        walk_events: root.walk_events(),
    };

    let out = format!("{holder:?}");
    assert!(out.contains("Ancestors { next: Some(Node { data: \"c1\""));
    assert!(out.contains("Children { first: Some(Node { data: \"c1\""));
    assert!(out.contains("last: Some(Node { data: \"c2\""));
    assert!(out.contains("Siblings { next: Some(Node { data: \"c1\""));
    assert!(out.contains("SkipTokens { iter: Children {"));
    assert!(out.contains("Walk { events: WalkEvents { next: Some((Next, Node { data: \"root\""));
    assert!(out.contains("WithDepths { events: WalkEvents {"));
    assert!(out.contains("Inside { events: WalkEvents {"));
    assert!(out.contains("depth: 0"));

    let mut it = root.walk_events();
    it.next();
    let out = format!("{it:?}");
    assert!(out.starts_with("WalkEvents { next: Some((Down, Node { data: \"c1\""));
    assert!(out.ends_with("depth: 1 }"));

    let mut it = c1.siblings();
    it.by_ref().for_each(drop);
    assert_eq!(format!("{it:?}"), "Siblings { next: None }");

    let empty = Holder::default();
    let out = format!("{empty:#?}");
    assert!(out.contains("Children {\n        first: None,\n        last: None,\n    }"));
    assert!(out.contains("Inside {\n        events: WalkEvents {\n            next: None,\n            depth: 0,\n        },\n        ..\n    }"));
    Ok(())
}

#[test]
fn size_hints() -> Result<(), Box<dyn core::error::Error>> {
    let tree = syntree::tree! {
        "root" => {
            "c1",
            "c2"
        }
    };

    let root = tree.first().ok_or("missing root")?;
    let c2 = root.last().ok_or("missing c2")?;

    assert_eq!(root.ancestors().size_hint(), (1, None));
    assert_eq!(c2.ancestors().size_hint(), (1, None));
    assert_eq!(root.children().size_hint(), (1, Some(3)));
    assert_eq!(c2.children().size_hint(), (0, Some(3)));
    assert_eq!(c2.siblings().size_hint(), (1, Some(3)));
    assert_eq!(root.walk_events().size_hint(), (1, None));

    let mut it = c2.siblings();
    it.next();
    assert_eq!(it.size_hint(), (0, Some(3)));

    assert_eq!(
        Ancestors::<S, FlavorDefault>::default().size_hint(),
        (0, None)
    );
    assert_eq!(
        Children::<S, FlavorDefault>::default().size_hint(),
        (0, Some(0))
    );
    assert_eq!(
        Siblings::<S, FlavorDefault>::default().size_hint(),
        (0, Some(0))
    );
    assert_eq!(
        WalkEvents::<S, FlavorDefault>::default().size_hint(),
        (0, None)
    );

    for (n, node) in [root, c2].into_iter().enumerate() {
        let a = node.ancestors();
        assert!(a.size_hint().0 <= a.count(), "ancestors {n}");
        let c = node.children();
        let (lo, hi) = c.size_hint();
        let count = c.count();
        assert!(
            lo <= count && hi.is_none_or(|hi| count <= hi),
            "children {n}"
        );
    }

    Ok(())
}
