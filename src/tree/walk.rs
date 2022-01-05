use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node};

/// An iterator that walks over the entire tree, visiting every node exactly
/// once.
///
/// See [Tree::walk][crate::Tree::walk].
pub struct Walk<'a, T> {
    tree: &'a [Links<T>],
    // The current node.
    start: Option<NonMaxUsize>,
    /// The terminating node that once visited we need to stop.
    term: Option<NonMaxUsize>,
}

impl<'a, T> Walk<'a, T> {
    pub(crate) fn new(
        tree: &'a [Links<T>],
        start: Option<NonMaxUsize>,
        term: Option<NonMaxUsize>,
    ) -> Self {
        Self { tree, start, term }
    }

    fn walk(&self, links: &Links<T>) -> Option<NonMaxUsize> {
        if let Some(id) = links.first.or(links.next) {
            return Some(id);
        }

        let mut links = links;

        while let Some(parent) = links.parent {
            links = self.tree.get(parent.get())?;

            if let Some(id) = links.next {
                if self.term == Some(id) {
                    return None;
                }

                return Some(id);
            }
        }

        None
    }
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.start.take()?;
        let links = self.tree.get(id.get())?;

        if let Some(id) = self.walk(links) {
            self.start = Some(id);
        }

        Some(Node::new(links, self.tree))
    }
}

/// An iterator that walks over the entire tree, visiting every node exactly
/// once.
///
/// See [Tree::walk_with_depths][crate::Tree::walk_with_depths].
pub struct WalkWithDepths<'a, T> {
    tree: &'a [Links<T>],
    // The current node.
    start: Option<NonMaxUsize>,
    /// The terminating node that once visited we need to stop.
    term: Option<NonMaxUsize>,
    /// The current depth being walked.
    depth: usize,
}

impl<'a, T> WalkWithDepths<'a, T> {
    pub(crate) fn new(
        tree: &'a [Links<T>],
        start: Option<NonMaxUsize>,
        term: Option<NonMaxUsize>,
    ) -> Self {
        Self {
            tree,
            start,
            term,
            depth: 0,
        }
    }

    fn walk(&mut self, links: &Links<T>) -> Option<NonMaxUsize> {
        if let Some(id) = links.first {
            self.depth = self.depth.checked_add(1)?;
            return Some(id);
        }

        if let Some(id) = links.next {
            return Some(id);
        }

        let mut links = links;

        while let Some(parent) = links.parent {
            self.depth = self.depth.checked_sub(1)?;
            links = self.tree.get(parent.get())?;

            if let Some(id) = links.next {
                if self.term == Some(id) {
                    return None;
                }

                return Some(id);
            }
        }

        None
    }
}

impl<'a, T> Iterator for WalkWithDepths<'a, T> {
    type Item = (usize, Node<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.start.take()?;
        let links = self.tree.get(id.get())?;
        let depth = self.depth;

        if let Some(id) = self.walk(links) {
            self.start = Some(id);
        }

        let node = Node::new(links, self.tree);
        Some((depth, node))
    }
}
