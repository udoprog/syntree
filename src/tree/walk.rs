use std::mem;

use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node};

/// An iterator that walks over the entire tree.
///
/// See [Tree::walk][crate::Tree::walk].
pub struct Walk<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    pub(crate) stack: Vec<(bool, NonMaxUsize)>,
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((down, top)) = self.stack.last_mut() {
            let links = self.tree.get(top.get())?;

            if mem::take(down) {
                if let Some(first) = links.first {
                    self.stack.push((true, first));
                }

                return Some(Node::new(links, self.tree));
            }

            let next = links.next;
            self.stack.pop();

            if let Some(next) = next {
                self.stack.push((true, next));
            }
        }

        None
    }
}
