use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node};

/// An iterator that walks over the entire tree.
///
/// See [Tree::walk][crate::Tree::walk].
pub struct Walk<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    // The terminating node.
    pub(crate) start: Option<NonMaxUsize>,
    pub(crate) end: Option<NonMaxUsize>,
}

impl<'a, T> Walk<'a, T> {
    fn forward(&mut self, links: &Links<T>, end: NonMaxUsize) -> Option<NonMaxUsize> {
        if let Some(next) = links.first.or(links.next) {
            return Some(next);
        }

        let mut step = links;

        loop {
            step = match step.parent {
                Some(parent) if parent != end => self.tree.get(parent.get())?,
                _ => break,
            };

            if let Some(next) = step.next {
                return Some(next);
            }
        }

        None
    }

    fn backward(&mut self, links: &Links<T>, start: NonMaxUsize) -> Option<NonMaxUsize> {
        if let Some(next) = links.last.or(links.prev) {
            return Some(next);
        }

        let mut step = links;

        loop {
            step = match step.parent {
                Some(parent) if parent != start => self.tree.get(parent.get())?,
                _ => break,
            };

            if let Some(prev) = step.prev {
                return Some(prev);
            }
        }

        None
    }
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = (self.start.take(), self.end.take());
        let (start, end) = (start?, end?);

        let links = self.tree.get(start.get())?;

        if start != end || links.first.is_some() {
            if let Some(next) = self.forward(links, end) {
                self.start = Some(next);
                self.end = Some(end);
            }
        }

        Some(Node::new(links, self.tree))
    }
}

impl<'a, T> DoubleEndedIterator for Walk<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (start, end) = (self.start.take(), self.end.take());
        let (start, end) = (start?, end?);

        let links = self.tree.get(end.get())?;

        if start != end || links.last.is_some() {
            if let Some(next) = self.backward(links, start) {
                self.start = Some(start);
                self.end = Some(next);
            }
        }

        Some(Node::new(links, self.tree))
    }
}
