use std::mem;

use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node};

/// An iterator that walks over the entire tree in reverse.
///
/// See [Tree::walk_rev][crate::Tree::walk_rev].
pub struct WalkRev<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    pub(crate) stack: Vec<(bool, NonMaxUsize)>,
}

impl<'a, T> Iterator for WalkRev<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((down, top)) = self.stack.last_mut() {
            let links = self.tree.get(top.get())?;

            if mem::take(down) {
                if let Some(last) = links.last {
                    self.stack.push((true, last));
                }

                return Some(Node {
                    node: links,
                    tree: self.tree,
                });
            }

            let prev = links.prev;
            self.stack.pop();

            if let Some(prev) = prev {
                self.stack.push((true, prev));
            }
        }

        None
    }
}
