use crate::tree::{Links, Node};

/// An iterator that walks over the entire tree.
///
/// See [Tree::walk][crate::Tree::walk].
pub struct Walk<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    // The terminating node.
    pub(crate) range: Option<(usize, usize)>,
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = self.range.take()?;
        let links = self.tree.get(start)?;

        if let Some(start) = start.checked_add(1) {
            if start <= end {
                self.range = Some((start, end));
            }
        }

        Some(Node::new(links, self.tree))
    }
}

impl<'a, T> DoubleEndedIterator for Walk<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (start, end) = self.range.take()?;
        let links = self.tree.get(end)?;

        if let Some(end) = end.checked_sub(1) {
            if start <= end {
                self.range = Some((start, end));
            }
        }

        Some(Node::new(links, self.tree))
    }
}
