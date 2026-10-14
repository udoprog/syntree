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

type Res = Result<(), Box<dyn core::error::Error>>;

fn values<'a>(it: impl IntoIterator<Item = syntree::Node<'a, S, FlavorDefault>>) -> Vec<S> {
    it.into_iter().map(|n| n.value()).collect()
}

fn val(n: Option<syntree::Node<'_, S, FlavorDefault>>) -> Option<S> {
    n.map(|n| n.value())
}

#[test]
fn children_interleaved_ends_meet() -> Res {
    let tree = syntree::tree! { "root" => { "a", "b", "c", "d" } };
    let root = tree.first().ok_or("missing root")?;

    let mut it = root.children();
    assert_eq!(val(it.next()), Some("a"));
    assert_eq!(val(it.next_back()), Some("d"));
    assert_eq!(val(it.next()), Some("b"));
    assert_eq!(val(it.next_back()), Some("c"));
    assert_eq!(val(it.next()), None);
    assert_eq!(val(it.next_back()), None);

    let mut it = root.children();
    assert_eq!(val(it.next()), Some("a"));
    assert_eq!(val(it.next_back()), Some("d"));
    assert_eq!(val(it.next_back()), Some("c"));
    assert_eq!(val(it.next()), Some("b"));
    assert_eq!(val(it.next()), None);
    assert_eq!(val(it.next_back()), None);

    let tree = syntree::tree! { "root" => { "a", "b", "c" } };
    let root = tree.first().ok_or("missing root")?;
    let mut it = root.children();
    assert_eq!(val(it.next()), Some("a"));
    assert_eq!(val(it.next_back()), Some("c"));
    assert_eq!(val(it.next()), Some("b"));
    assert_eq!(val(it.next_back()), None);
    assert_eq!(val(it.next()), None);
    Ok(())
}

#[test]
fn children_exhaust_one_end_then_other() -> Res {
    let tree = syntree::tree! { "root" => { "a", "b", "c" } };
    let root = tree.first().ok_or("missing root")?;

    let mut it = root.children();
    assert_eq!(values(it.by_ref()), ["a", "b", "c"]);
    assert_eq!(val(it.next_back()), None);
    assert_eq!(val(it.next()), None);

    let mut it = root.children();
    assert_eq!(values(it.by_ref().rev()), ["c", "b", "a"]);
    assert_eq!(val(it.next()), None);
    assert_eq!(val(it.next_back()), None);

    assert_eq!(values(tree.children().rev()), ["root"]);
    Ok(())
}

#[test]
fn children_single_child() -> Res {
    let tree = syntree::tree! { "root" => { "a" } };
    let root = tree.first().ok_or("missing root")?;

    let mut it = root.children();
    assert_eq!(val(it.next()), Some("a"));
    assert_eq!(val(it.next_back()), None);
    assert_eq!(val(it.next()), None);

    let mut it = root.children();
    assert_eq!(val(it.next_back()), Some("a"));
    assert_eq!(val(it.next()), None);
    assert_eq!(val(it.next_back()), None);
    Ok(())
}

#[test]
fn empty_tree_iterators() -> Res {
    let tree = syntree::Builder::<S>::new().build()?;
    assert_eq!(tree.walk().count(), 0);
    assert_eq!(tree.walk().inside().count(), 0);
    assert_eq!(tree.walk().with_depths().count(), 0);
    assert_eq!(tree.children().count(), 0);
    assert_eq!(tree.children().rev().count(), 0);
    assert_eq!(tree.walk_events().count(), 0);
    assert_eq!(tree.walk_events().depth(), 0);
    assert!(tree.first().is_none());
    assert!(tree.last().is_none());
    Ok(())
}

fn assert_fused<I: Iterator>(mut it: I) {
    for _ in 0..3 {
        assert!(it.next().is_none());
    }
}

#[test]
fn default_iterators_are_empty() {
    assert_fused(Ancestors::<S, FlavorDefault>::default());
    assert_fused(Children::<S, FlavorDefault>::default());
    assert_fused(Children::<S, FlavorDefault>::default().rev());
    assert_fused(Siblings::<S, FlavorDefault>::default());
    assert_fused(SkipTokens::<Children<'_, S, FlavorDefault>>::default());
    assert_fused(SkipTokens::<Children<'_, S, FlavorDefault>>::default().rev());
    assert_fused(Walk::<S, FlavorDefault>::default());
    assert_fused(WithDepths::<S, FlavorDefault>::default());
    assert_fused(Inside::<S, FlavorDefault>::default());
    assert_fused(WalkEvents::<S, FlavorDefault>::default());
    assert_eq!(WalkEvents::<S, FlavorDefault>::default().depth(), 0);
}

#[test]
fn exhausted_iterators_stay_exhausted() -> Res {
    fn drain<I: Iterator>(mut it: I) {
        it.by_ref().for_each(drop);
        assert_fused(it);
    }

    let tree = syntree::tree! {
        "root" => {
            "a" => { ("t", 1) },
            "b"
        },
        "root2"
    };
    let root = tree.first().ok_or("missing root")?;
    let a = root.first().ok_or("missing a")?;

    drain(a.ancestors());
    drain(root.children());
    drain(root.children().rev());
    drain(a.siblings());
    drain(root.children().skip_tokens());
    drain(tree.walk());
    drain(root.walk());
    drain(root.walk().with_depths());
    drain(root.walk().inside());
    drain(tree.walk().inside());
    drain(root.walk_events());
    drain(tree.walk_events());
    Ok(())
}

#[test]
fn walk_from_ends_of_tree() -> Res {
    let tree = syntree::tree! {
        "root" => {
            "a",
            "b" => { "b1" }
        },
        "root2" => {
            "c" => { "e" }
        }
    };
    let root = tree.first().ok_or("missing root")?;
    let a = root.first().ok_or("missing a")?;
    let b1 = root.last().and_then(|b| b.first()).ok_or("missing b1")?;
    let root2 = tree.last().ok_or("missing root2")?;
    let e = root2.first().and_then(|c| c.first()).ok_or("missing e")?;

    assert_eq!(values(a.walk_from()), ["b", "b1", "root2", "c", "e"]);
    assert_eq!(values(b1.walk_from()), ["root2", "c", "e"]);
    assert_eq!(values(root.walk_from()), ["root2", "c", "e"]);
    assert_eq!(values(root2.walk_from()), [] as [S; 0]);
    assert_eq!(values(e.walk_from()), [] as [S; 0]);
    Ok(())
}

#[test]
fn walk_events_depth_after_full_walk() -> Res {
    use syntree::node::Event::{Down, Next, Up};

    let tree = syntree::tree! {
        "root" => {
            "a" => { "a1" }
        },
        "root2"
    };

    let mut it = tree.walk_events();
    let events = it.by_ref().map(|(e, n)| (e, n.value())).collect::<Vec<_>>();
    assert_eq!(
        events,
        [
            (Next, "root"),
            (Down, "a"),
            (Down, "a1"),
            (Up, "a"),
            (Up, "root"),
            (Next, "root2"),
        ]
    );
    // Leaving the last root decrements depth before finding no parent, so an
    // exhausted tree walk reports -1 rather than 0.
    assert_eq!(it.depth(), -1);

    let root = tree.first().ok_or("missing root")?;
    let mut it = root.walk_events();
    let mut depths = Vec::new();

    while let Some((e, n)) = it.next() {
        depths.push((e, n.value(), it.depth()));
    }

    assert_eq!(
        depths,
        [
            (Next, "root", 1),
            (Down, "a", 2),
            (Down, "a1", 1),
            (Up, "a", 0),
            (Up, "root", 0),
            (Next, "root2", -1),
        ]
    );
    Ok(())
}

#[test]
fn with_depths_escapes_subtree() -> Res {
    let tree = syntree::tree! {
        "root" => {
            "a",
            "b" => { "b1" },
            "c"
        },
        "root2" => { "d" }
    };
    let b = tree
        .first()
        .and_then(|r| r.first())
        .and_then(|a| a.next())
        .ok_or("missing b")?;

    let depths = b
        .walk()
        .with_depths()
        .map(|(d, n)| (d, n.value()))
        .collect::<Vec<_>>();
    assert_eq!(
        depths,
        [(0, "b"), (1, "b1"), (0, "c"), (-1, "root2"), (0, "d")]
    );

    let mut walk = b.walk();
    assert_eq!(
        walk.next_with_depth().map(|(d, n)| (d, n.value())),
        Some((0, "b"))
    );
    let rest = walk
        .with_depths()
        .map(|(d, n)| (d, n.value()))
        .collect::<Vec<_>>();
    assert_eq!(rest, [(1, "b1"), (0, "c"), (-1, "root2"), (0, "d")]);
    Ok(())
}

#[test]
fn inside_from_various_starts() -> Res {
    let tree = syntree::tree! {
        "root" => {
            ("tok", 1),
            "b" => { "b1" => { "b2" }, "b3" },
            "c"
        },
        "root2"
    };
    let root = tree.first().ok_or("missing root")?;
    let tok = root.first().ok_or("missing tok")?;
    let b = tok.next().ok_or("missing b")?;

    assert_eq!(values(tok.walk().inside()), ["tok"]);
    assert_eq!(values(b.walk().inside()), ["b", "b1", "b2", "b3"]);
    assert_eq!(
        values(root.walk().inside()),
        ["root", "tok", "b", "b1", "b2", "b3", "c"]
    );

    let mut walk = b.walk();
    assert_eq!(values(walk.by_ref().take(3)), ["b", "b1", "b2"]);
    assert_eq!(values(walk.inside()), ["b3"]);
    Ok(())
}

#[test]
fn skip_tokens_double_ended() -> Res {
    let tree = syntree::tree! {
        "root" => {
            ("t1", 1),
            "a" => { ("x", 1) },
            ("t2", 1),
            "empty" => {},
            "b" => { ("y", 1) },
            ("t3", 1)
        }
    };
    let root = tree.first().ok_or("missing root")?;

    assert_eq!(values(root.children().skip_tokens()), ["a", "b"]);
    assert_eq!(values(root.children().skip_tokens().rev()), ["b", "a"]);

    let mut it = root.children().skip_tokens();
    assert_eq!(val(it.next_back()), Some("b"));
    assert_eq!(val(it.next()), Some("a"));
    assert_eq!(val(it.next_back()), None);
    assert_eq!(val(it.next()), None);

    let mut it = root.children().skip_tokens();
    assert_eq!(val(it.rfind(|n| n.value() == "a")), Some("a"));
    assert_eq!(val(it.next()), None);

    let mut it = root.children().skip_tokens();
    assert_eq!(val(it.rfind(|n| n.value() == "t3")), None);
    assert_eq!(val(it.next()), None);

    let mut it = root.children().skip_tokens();
    assert_eq!(val(it.find(|n| n.value() == "b")), Some("b"));
    assert_eq!(val(it.next_back()), None);
    Ok(())
}

#[test]
fn next_node_skips_tokens() -> Res {
    let tree = syntree::tree! {
        "root" => {
            ("t1", 1),
            "a" => {
                "inner" => { ("x", 1) }
            },
            ("t2", 1),
            "b" => { ("y", 1) }
        }
    };
    let root = tree.first().ok_or("missing root")?;
    let t1 = root.first().ok_or("missing t1")?;
    let x = root.walk().find(|n| n.value() == "x").ok_or("missing x")?;

    let mut it = root.children();
    assert_eq!(val(it.next_node()), Some("a"));
    assert_eq!(val(it.next()), Some("t2"));
    assert_eq!(val(it.next_node()), Some("b"));
    assert_eq!(val(it.next_node()), None);

    let mut it = t1.siblings();
    assert_eq!(val(it.next_node()), Some("a"));
    assert_eq!(val(it.next_node()), Some("b"));
    assert_eq!(val(it.next_node()), None);

    let mut it = x.ancestors();
    assert_eq!(val(it.next_node()), Some("inner"));
    assert_eq!(val(it.next_node()), Some("a"));
    assert_eq!(val(it.next_node()), Some("root"));
    assert_eq!(val(it.next_node()), None);

    assert_eq!(values(x.ancestors().skip_tokens()), ["inner", "a", "root"]);
    assert_eq!(values(t1.siblings().skip_tokens()), ["a", "b"]);
    assert_eq!(
        values(root.walk().skip_tokens()),
        ["root", "a", "inner", "b"]
    );
    Ok(())
}

#[test]
fn find_preceding_edge_cases() -> Res {
    let tree = syntree::tree! {
        "root" => {
            "p" => { "q" => { ("t", 1) } },
            "m",
            "n" => { "target" }
        }
    };
    let root = tree.first().ok_or("missing root")?;
    let p = root.first().ok_or("missing p")?;
    let n = root.last().ok_or("missing n")?;
    let target = n.first().ok_or("missing target")?;

    assert_eq!(val(root.find_preceding(|_| true)), None);
    assert_eq!(val(p.find_preceding(|_| true)), None);

    // Only the nearest previous sibling at each level is tested, so `p` behind
    // the non-matching `m` is never reached.
    assert_eq!(val(target.find_preceding(|n| n.value() == "p")), None);

    assert_eq!(val(target.find_preceding(|n| n.value() == "m")), Some("m"));
    assert_eq!(val(n.find_preceding(|_| true)), Some("m"));

    let tree = syntree::tree! {
        "root" => {
            "p" => { "q" => { ("t", 1) } },
            "n" => { "target" }
        }
    };
    let target = tree
        .first()
        .and_then(|r| r.last())
        .and_then(|n| n.first())
        .ok_or("missing target")?;

    assert_eq!(val(target.find_preceding(|_| true)), Some("t"));
    assert_eq!(val(target.find_preceding(|n| n.has_children())), Some("q"));
    assert_eq!(val(target.find_preceding(|n| n.value() == "p")), Some("p"));
    Ok(())
}

#[test]
fn clone_continues_independently() -> Res {
    let tree = syntree::tree! {
        "root" => {
            "a" => { "a1" },
            "b",
            "c"
        },
        "root2"
    };
    let root = tree.first().ok_or("missing root")?;
    let a = root.first().ok_or("missing a")?;

    let mut walk = tree.walk();
    walk.next();
    walk.next();
    let copy = walk.clone();
    assert_eq!(values(walk.by_ref().take(2)), ["a1", "b"]);
    assert_eq!(values(copy.clone()), ["a1", "b", "c", "root2"]);
    assert_eq!(values(copy.inside()), ["a1", "b", "c", "root2"]);
    assert_eq!(values(walk), ["c", "root2"]);

    let mut inside = root.walk().inside();
    inside.next();
    let copy = inside.clone();
    assert_eq!(values(inside), ["a", "a1", "b", "c"]);
    assert_eq!(values(copy), ["a", "a1", "b", "c"]);

    let mut depths = root.walk().with_depths();
    depths.next();
    depths.next();
    let copy = depths.clone();
    assert_eq!(
        depths.map(|(d, n)| (d, n.value())).collect::<Vec<_>>(),
        [(2, "a1"), (1, "b"), (1, "c"), (0, "root2")]
    );
    assert_eq!(copy.count(), 4);

    let mut events = root.walk_events();
    events.next();
    events.next();
    let copy = events.clone();
    assert_eq!(copy.depth(), events.depth());
    assert_eq!(events.count(), copy.count());

    let mut children = root.children();
    children.next();
    let mut copy = children.clone();
    assert_eq!(val(children.next_back()), Some("c"));
    assert_eq!(values(copy.by_ref().rev()), ["c", "b"]);
    assert_eq!(values(children), ["b"]);

    let mut siblings = a.siblings();
    siblings.next();
    let copy = siblings.clone();
    assert_eq!(values(siblings), ["b", "c"]);
    assert_eq!(values(copy), ["b", "c"]);

    let a1 = a.first().ok_or("missing a1")?;
    let mut ancestors = a1.ancestors();
    ancestors.next();
    let copy = ancestors.clone();
    assert_eq!(values(ancestors), ["a", "root"]);
    assert_eq!(values(copy), ["a", "root"]);

    let mut skip = root.children().skip_tokens();
    assert_eq!(val(skip.next()), Some("a"));
    let copy = skip.clone();
    assert_eq!(skip.count(), 0);
    assert_eq!(copy.count(), 0);
    Ok(())
}
