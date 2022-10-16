use std::iter::FusedIterator;

use crate::links::Links;
use crate::{Kind, Node, SkipTokens};

/// An iterator that iterates over the [Node::next] elements of a node. This is
/// typically used for iterating over the children of a tree.
///
/// Note that this iterator also implements [Default], allowing it to
/// effectively create an empty iterator in case a particular sibling is not
/// available:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2"
///         },
///         "child3"
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.next()).map(|n| n.siblings()).unwrap_or_default();
/// assert_eq!(it.next().map(|n| *n.value()), None);
/// # Ok(()) }
/// ```
///
/// See [Node::siblings].
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2"
///         },
///         "child3"
///     },
///     "root2" => {
///         "child4"
///     }
/// };
///
/// let root = tree.first().ok_or("missing root")?;
///
/// assert_eq!(
///     root.siblings().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
/// # Ok(()) }
/// ```
pub struct Siblings<'a, T> {
    tree: &'a [Links<T>],
    links: Option<&'a Links<T>>,
}

impl<'a, T> Siblings<'a, T> {
    /// Construct a new child iterator.
    pub(crate) const fn new(tree: &'a [Links<T>], links: &'a Links<T>) -> Self {
        Self {
            tree,
            links: Some(links),
        }
    }

    /// Construct a [SkipTokens] iterator from the remainder of this
    /// iterator. This filters out [Kind::Token] elements.
    ///
    /// See [SkipTokens] for documentation.
    pub const fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }

    /// Get the next node from the iterator. This advances past all non-node
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     ("t1", 1),
    ///     "child1",
    ///     ("t2", 1),
    ///     "child2",
    ///     ("t3", 1),
    ///     "child3",
    ///     ("t4", 1)
    /// };
    ///
    /// let first = tree.first().ok_or("missing first")?;
    ///
    /// let mut it = first.siblings();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(*n.value());
    /// }
    ///
    /// assert_eq!(out, ["child1", "child2", "child3"]);
    /// # Ok(()) }
    /// ```
    pub fn next_node(&mut self) -> Option<Node<'a, T>> {
        loop {
            let node = self.next()?;

            if matches!(node.kind(), Kind::Node) {
                return Some(node);
            }
        }
    }
}

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let links = self.links.take()?;
        self.links = links.next.and_then(|id| self.tree.get(id.get()));
        Some(Node::new(links, self.tree))
    }
}

impl<'a, T> FusedIterator for Siblings<'a, T> {}

impl<'a, T> Clone for Siblings<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Siblings<'a, T> {}

impl<'a, T> Default for Siblings<'a, T> {
    fn default() -> Self {
        Self {
            tree: &[],
            links: None,
        }
    }
}