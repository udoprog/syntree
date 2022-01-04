use std::collections::VecDeque;
use std::mem::{self, replace};

use crate::builder::{self, Links};
use crate::non_max::NonMaxUsize;
use crate::tree;

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree<T>(b: &builder::TreeBuilder<T>) -> tree::Tree<T>
where
    T: Clone,
{
    let mut tree = Vec::<tree::Links<T>>::new();
    let mut last = None;

    let mut queue = VecDeque::new();
    let mut children = Vec::new();

    let mut cur = b.get(0);

    while let Some(c) = cur.take() {
        queue.push_back((0usize, c, None::<NonMaxUsize>));
        cur = c.next.and_then(|id| b.get(id.get()));
    }

    // Stack of previous variables.
    let mut stack = Vec::new();

    while let Some((depth, el, parent)) = queue.pop_front() {
        let id = NonMaxUsize::new(tree.len()).expect("ran out of ids");
        let new_depth = depth.checked_add(1).expect("depth out of bounds");

        if parent.is_none() {
            // The last top-level item in the tree.
            last = Some(id);
        }

        // First node is always id + 1 with the specific layout. So all we need
        // to do is to check that the node actually has a child.
        let first = if el.first.is_some() {
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

        stack.resize(new_depth, None);
        let prev = replace(&mut stack[depth], Some(id));

        tree.push(tree::Links {
            data: el.data.clone(),
            kind: el.kind,
            prev,
            next: None,
            first,
            last: None,
        });

        if let Some(node) = prev.and_then(|id| tree.get_mut(id.get())) {
            node.next = Some(id);
        }

        let mut cur = el.first.and_then(|id| b.get(id.get()));

        while let Some(c) = cur.take() {
            children.push(c);
            cur = c.next.and_then(|id| b.get(id.get()));
        }

        for el in children.drain(..).rev() {
            let depth = depth.checked_add(1).expect("depth overflow");
            queue.push_front((depth, el, Some(id)));
        }
    }

    tree::Tree::new(tree, last)
}

struct Step<'a, T> {
    links: &'a Links<T>,
    down: bool,
    depth: usize,
    parent: Option<NonMaxUsize>,
}

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree2<T>(b: &builder::TreeBuilder<T>) -> tree::Tree<T>
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

    while let Some(Step {
        links,
        down,
        depth,
        parent,
    }) = stack.last_mut()
    {
        let links = *links;
        let depth = *depth;
        let parent = *parent;

        if !mem::take(down) {
            stack.pop();

            if let Some(links) = links.next.and_then(|id| b.get(id.get())) {
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

        if let Some(first) = links.first.and_then(|id| b.get(id.get())) {
            stack.push(Step {
                links: first,
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
        let first = if links.first.is_some() {
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
        let prev = replace(&mut back[depth], Some(id));

        tree.push(tree::Links {
            data: links.data.clone(),
            kind: links.kind,
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
