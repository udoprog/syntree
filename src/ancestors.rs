use std::iter::FusedIterator;

use crate::{Kind, Node, SkipTokens};

/// An iterator that iterates over the [Node::parent] elements of a node. This
/// is used for iterating over the ancestors of a node.
///
/// Note that this iterator also implements [Default], allowing it to
/// effectively create an empty iterator in case a particular ancestor is not
/// available:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1",
///         "child3"
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.first()).and_then(|n| n.first()).map(|n| n.ancestors()).unwrap_or_default();
/// assert!(it.next().is_none());
/// # Ok(()) }
/// ```
///
/// See [Node::ancestors].
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
///     }
/// };
///
/// let child2 = tree.first().and_then(|n| n.first()).and_then(|n| n.first()).ok_or("missing child2")?;
/// assert_eq!(*child2.value(), "child2");
///
/// assert_eq!(
///     child2.ancestors().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["child2", "child1", "root"]
/// );
/// # Ok(()) }
/// ```
pub struct Ancestors<'a, T> {
    node: Option<Node<'a, T>>,
}

impl<'a, T> Ancestors<'a, T> {
    /// Construct a new ancestor iterator.
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
    ///     "root" => {
    ///         "child" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// let lit = tree.first().and_then(|n| n.first()).and_then(|n| n.first()).ok_or("missing lit")?;
    /// assert_eq!(*lit.value(), "lit");
    ///
    /// let mut it = lit.ancestors();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(*n.value());
    /// }
    ///
    /// assert_eq!(out, ["child", "root"]);
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

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.take()?;
        self.node = node.parent();
        Some(node)
    }
}

impl<'a, T> FusedIterator for Ancestors<'a, T> {}

impl<'a, T> Clone for Ancestors<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Ancestors<'a, T> {}

impl<'a, T> Default for Ancestors<'a, T> {
    fn default() -> Self {
        Self { node: None }
    }
}
