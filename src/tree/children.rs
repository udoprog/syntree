use crate::non_max::NonMaxUsize;
use crate::tree::{Kind, Links, Node};

/// Iterator over the children of a node or tree. This excludes [Kind::Token]
/// nodes.
///
/// See [Tree::children][crate::Tree::children].
pub struct Children<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    pub(crate) start: Option<NonMaxUsize>,
    pub(crate) end: Option<NonMaxUsize>,
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.tree.get(self.start?.get())?;
            self.start = node.next;

            if matches!(node.kind, Kind::Token(..)) {
                continue;
            }

            return Some(Node {
                node,
                tree: self.tree,
            });
        }
    }
}

impl<'a, T> DoubleEndedIterator for Children<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.tree.get(self.end?.get())?;
            self.end = node.prev;

            if matches!(node.kind, Kind::Token(..)) {
                continue;
            }

            return Some(Node {
                node,
                tree: self.tree,
            });
        }
    }
}

impl<'a, T> Clone for Children<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Children<'a, T> {}
