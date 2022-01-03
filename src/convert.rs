use std::collections::VecDeque;
use std::fmt;
use std::mem::replace;

use crate::builder;
use crate::tree;

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree<T>(b: &builder::TreeBuilder<T>) -> tree::Tree<T>
where
    T: fmt::Debug + Copy,
{
    let mut tree = Vec::<tree::Internal<T>>::new();
    let mut last = usize::MAX;

    let mut queue = VecDeque::new();
    let mut children = Vec::new();

    let mut cur = b.data.get(0);

    while let Some(c) = cur.take() {
        queue.push_back((0usize, c, usize::MAX));
        cur = b.data.get(c.next);
    }

    // Stack of previous variables.
    let mut stack = Vec::new();

    while let Some((depth, el, parent)) = queue.pop_front() {
        let id = tree.len();

        if parent == usize::MAX {
            // The last top-level item in the tree.
            last = id;
        }

        // First node is always id + 1 with the specific layout. So all we need
        // to do is to check that the node actually has a child.
        let first = if el.first != usize::MAX {
            id + 1
        } else {
            usize::MAX
        };

        if let Some(parent) = tree.get_mut(parent) {
            parent.last = id;
        }

        stack.resize(depth + 1, usize::MAX);
        let prev = replace(&mut stack[depth], id);

        tree.push(tree::Internal {
            data: el.data,
            kind: el.kind,
            prev,
            next: usize::MAX,
            first,
            last: usize::MAX,
        });

        if let Some(node) = tree.get_mut(prev) {
            node.next = id;
        }

        let mut cur = b.data.get(el.first);

        while let Some(c) = cur.take() {
            children.push(c);
            cur = b.data.get(c.next);
        }

        for el in children.drain(..).rev() {
            queue.push_front((depth + 1, el, id));
        }
    }

    tree::Tree::new(tree, last)
}
