use std::mem;

use crate::builder;
use crate::tree;

#[derive(Clone, Copy)]
struct Params {
    down: bool,
    parent: usize,
    prev: usize,
}

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree<T>(builder: &builder::TreeBuilder<T>) -> tree::Tree<T>
where
    T: Copy,
{
    let mut stack = Vec::new();

    let params = Params {
        down: true,
        parent: usize::MAX,
        prev: usize::MAX,
    };

    stack.push((0, params));

    let mut tree = Vec::new();
    let mut last = usize::MAX;

    while let Some((id, params)) = stack.last().copied() {
        let cur = match builder.get(id) {
            Some(cur) => cur,
            None => break,
        };

        // Navigate to sibling.
        if !params.down {
            if let Some(n) = cur.next {
                stack.pop();
                stack.push((
                    n,
                    Params {
                        down: true,
                        prev: id,
                        ..params
                    },
                ));
            } else {
                stack.pop();
            }

            continue;
        }

        let id = tree.len();

        tree.push(tree::Internal::new(cur.data, cur.kind, params.prev));

        if let Some(n) = tree.get_mut(params.prev) {
            *n.next_mut() = id;
        }

        if let Some(n) = tree.get_mut(params.parent) {
            if n.first() == usize::MAX {
                *n.first_mut() = id;
            }

            *n.last_mut() = id;
        } else {
            last = id;
        }

        if let Some(n) = cur.child {
            stack.push((
                n,
                Params {
                    down: true,
                    parent: id,
                    prev: usize::MAX,
                },
            ));
            continue;
        }

        // No longer crawling down, so disable it for the current stack.
        for (_, params) in stack.iter_mut().rev() {
            if !mem::take(&mut params.down) {
                break;
            }
        }
    }

    tree::Tree::new(tree, last)
}
