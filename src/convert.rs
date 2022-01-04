use std::collections::VecDeque;
use std::fmt;
use std::mem::replace;

use crate::builder;
use crate::non_max::NonMaxUsize;
use crate::tree;

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree<T>(b: &builder::TreeBuilder<T>) -> tree::Tree<T>
where
    T: fmt::Debug + Copy,
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

        stack.resize(depth + 1, None);
        let prev = replace(&mut stack[depth], Some(id));

        tree.push(tree::Links {
            data: el.data,
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
