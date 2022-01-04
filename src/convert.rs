use std::mem;

use crate::builder::{self, Links};
use crate::non_max::NonMaxUsize;
use crate::tree;

struct Step<'a, T> {
    links: &'a Links<T>,
    down: bool,
    depth: usize,
    parent: Option<NonMaxUsize>,
}

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree<T>(b: &builder::TreeBuilder<T>) -> tree::Tree<T>
where
    T: Clone,
{
    let mut tree = Vec::<tree::Links<T>>::new();
    let mut last = None;

    // Stack of previous variables.
    let mut back = Vec::new();

    // Stack used to walk.
    let mut stack = b
        .get(0)
        .map(|links| Step {
            links,
            down: true,
            depth: 0,
            parent: None,
        })
        .into_iter()
        .collect::<Vec<_>>();

    while let Some(step) = stack.last_mut() {
        let depth = step.depth;
        let parent = step.parent;

        if !mem::take(&mut step.down) {
            let next = step.links.next;
            stack.pop();

            if let Some(links) = next.and_then(|id| b.get(id.get())) {
                stack.push(Step {
                    links,
                    down: true,
                    depth,
                    parent,
                });
            }

            continue;
        }

        let id = NonMaxUsize::new(tree.len()).expect("ran out of ids");
        let new_depth = depth.checked_add(1).expect("depth out of bounds");
        let data = step.links.data.clone();
        let kind = step.links.kind;
        let first = step.links.first;

        if let Some(links) = first.and_then(|id| b.get(id.get())) {
            stack.push(Step {
                links,
                down: true,
                depth: new_depth,
                parent: Some(id),
            });
        }

        if parent.is_none() {
            // The last top-level item in the tree.
            last = Some(id);
        }

        // First node is always id + 1 with the specific layout. So all we need
        // to do is to check that the node actually has a child.
        let first = if first.is_some() {
            let id = id
                .get()
                .checked_add(1)
                .and_then(NonMaxUsize::new)
                .expect("id overflow");

            Some(id)
        } else {
            None
        };

        if let Some(parent) = parent.and_then(|id| tree.get_mut(id.get())) {
            parent.last = Some(id);
        }

        back.resize(new_depth, None);
        let prev = mem::replace(&mut back[depth], Some(id));

        tree.push(tree::Links {
            data,
            kind,
            prev,
            next: None,
            first,
            last: None,
        });

        if let Some(node) = prev.and_then(|id| tree.get_mut(id.get())) {
            node.next = Some(id);
        }
    }

    tree::Tree::new(tree, last)
}
