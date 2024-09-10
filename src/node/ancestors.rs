use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::node::{Node, SkipTokens};

/// An iterator that iterates over the [`Node::parent`] elements of a node. This
/// is used for iterating over the ancestors of a node.
///
/// Note that this iterator also implements [Default], allowing it to
/// effectively create an empty iterator in case a particular ancestor is not
/// available:
///
/// ```
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {},
///         "child3" => {}
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.first()).and_then(|n| n.first()).map(|n| n.ancestors()).unwrap_or_default();
/// assert!(it.next().is_none());
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// See [`Node::ancestors`].
///
/// # Examples
///
/// ```
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2" => {}
///         },
///         "child3" => {}
///     }
/// };
///
/// let child2 = tree.first().and_then(|n| n.first()).and_then(|n| n.first()).ok_or("missing child2")?;
/// assert_eq!(child2.value(), "child2");
///
/// assert_eq!(
///     child2.ancestors().map(|n| n.value()).collect::<Vec<_>>(),
///     ["child2", "child1", "root"]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Ancestors<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    node: Option<Node<'a, T, F>>,
}

impl<'a, T, F> Ancestors<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new ancestor iterator.
    #[inline]
    pub(crate) const fn new(node: Option<Node<'a, T, F>>) -> Self {
        Self { node }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this iterator.
    /// This filters out childless nodes, also known as tokens.
    ///
    /// See [`SkipTokens`] for documentation.
    #[inline]
    #[must_use]
    pub const fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }

    /// Get the next node from the iterator. This advances past all non-node
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// let lit = tree.first().and_then(|n| n.first()?.first()).ok_or("missing lit")?;
    /// assert_eq!(lit.value(), "lit");
    ///
    /// let mut it = lit.ancestors();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(n.value());
    /// }
    ///
    /// assert_eq!(out, ["child", "root"]);
    ///
    /// let mut it = lit.ancestors();
    ///
    /// let child = it.next_node().ok_or("missing child")?;
    /// let root = it.next_node().ok_or("missing root")?;
    ///
    /// assert_eq!([child.value(), root.value()], ["child", "root"]);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn next_node(&mut self) -> Option<Node<'a, T, F>> {
        self.find(|n| n.has_children())
    }
}

impl<'a, T, F> Iterator for Ancestors<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Node<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.take()?;
        self.node = node.parent();
        Some(node)
    }
}

impl<T, F> FusedIterator for Ancestors<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for Ancestors<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T, F> Default for Ancestors<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self { node: None }
    }
}
