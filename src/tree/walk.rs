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
    // The terminating node that once visited we need to stop.
    term: Option<NonMaxUsize>,
    // Parent nodes.
    parents: Vec<NonMaxUsize>,
}

impl<'a, T> Walk<'a, T> {
    pub(crate) fn new(
        tree: &'a [Links<T>],
        start: Option<NonMaxUsize>,
        term: Option<NonMaxUsize>,
    ) -> Self {
        Self {
            tree,
            start,
            term,
            parents: Vec::new(),
        }
    }

    fn walk(&mut self, id: NonMaxUsize, links: &Links<T>) -> Option<NonMaxUsize> {
        if let Some(first) = links.first {
            self.parents.push(id);
            return Some(first);
        }

        if let Some(next) = links.next {
            return Some(next);
        }

        while let Some(parent) = self.parents.pop() {
            if let Some(id) = self.tree.get(parent.get())?.next {
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

        if let Some(id) = self.walk(id, links) {
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
    parents: Vec<NonMaxUsize>,
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
            parents: Vec::new(),
        }
    }

    fn walk(&mut self, id: NonMaxUsize, links: &Links<T>) -> Option<NonMaxUsize> {
        if let Some(first) = links.first {
            self.parents.push(id);
            return Some(first);
        }

        if let Some(next) = links.next {
            return Some(next);
        }

        while let Some(parent) = self.parents.pop() {
            if let Some(id) = self.tree.get(parent.get())?.next {
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
        let depth = self.parents.len();
        let links = self.tree.get(id.get())?;

        if let Some(id) = self.walk(id, links) {
            self.start = Some(id);
        }

        let node = Node::new(links, self.tree);
        Some((depth, node))
    }
}
