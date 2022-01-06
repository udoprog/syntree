use std::iter::FusedIterator;

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
/// See [Tree::children][crate::Tree::children], [Node::children], or
/// [Node::siblings].
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
/// assert_eq!(
///     tree.children().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
///
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.children().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["child1", "child3"]
/// );
///
/// assert_eq!(
///     root.siblings().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
/// # Ok(()) }
/// ```
pub struct Nodes<'a, T> {
    node: Option<Node<'a, T>>,
}

impl<'a, T> Nodes<'a, T> {
    /// Construct a new child iterator.
    pub(crate) const fn new(node: Option<Node<'a, T>>) -> Self {
        Self { node }
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
    /// let mut it = tree.children();
    /// assert_eq!(it.next_node().map(|n| *n.value()), Some("child1"));
    /// assert_eq!(it.next().map(|n| *n.value()), Some("t2"));
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

impl<'a, T> Iterator for Nodes<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.take()?;
        self.node = node.next();
        Some(node)
    }
}

impl<'a, T> FusedIterator for Nodes<'a, T> {}

impl<'a, T> Clone for Nodes<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Nodes<'a, T> {}

impl<'a, T> Default for Nodes<'a, T> {
    fn default() -> Self {
        Self { node: None }
    }
}
