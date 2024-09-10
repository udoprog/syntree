use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::links::Links;
use crate::node::{Node, SkipTokens};
use crate::pointer::Pointer;

/// An iterator that iterates over the [`Node::next`] elements of a node. This is
/// typically used for iterating over the children of a tree.
///
/// Note that this iterator also implements [Default], allowing it to
/// effectively create an empty iterator in case a particular sibling is not
/// available:
///
/// ```
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2" => {
///                 "token1"
///             }
///         },
///         "child3" => {}
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.last()).map(|n| n.children()).unwrap_or_default();
/// assert_eq!(it.next().map(|n| n.value()), None);
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// See [`Tree::children`][crate::Tree::children] or [`Node::children`].
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
///     },
///     "root2" => {
///         "child4" => {}
///     }
/// };
///
/// assert_eq!(
///     tree.children().map(|n| n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
///
/// assert_eq!(
///     tree.children().rev().map(|n| n.value()).collect::<Vec<_>>(),
///     ["root2", "root"]
/// );
///
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.children().map(|n| n.value()).collect::<Vec<_>>(),
///     ["child1", "child3"]
/// );
///
/// assert_eq!(
///     root.children().rev().map(|n| n.value()).collect::<Vec<_>>(),
///     ["child3", "child1"]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    tree: &'a [Links<T, F::Index, F::Pointer>],
    first: Option<F::Pointer>,
    last: Option<F::Pointer>,
}

impl<'a, T, F> Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new child iterator.
    #[inline]
    pub(crate) const fn new(
        tree: &'a [Links<T, F::Index, F::Pointer>],
        first: Option<F::Pointer>,
        last: Option<F::Pointer>,
    ) -> Self {
        Self { tree, first, last }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this iterator.
    /// This filters out childless nodes, also known as tokens.
    ///
    /// See [`SkipTokens`] for documentation.
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
    ///     ("token1", 1),
    ///     "child1" => {
    ///         "token2"
    ///     },
    ///     ("token3", 1),
    ///     "child2" => {
    ///         "token4"
    ///     },
    ///     ("token5", 1),
    ///     "child3" => {
    ///         "token6"
    ///     },
    ///     ("token7", 1)
    /// };
    ///
    /// let mut it = tree.children();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(n.value());
    /// }
    ///
    /// assert_eq!(out, ["child1", "child2", "child3"]);
    ///
    /// let mut it = tree.children();
    ///
    /// let c1 = it.next_node().ok_or("missing child1")?;
    /// let c2 = it.next_node().ok_or("missing child2")?;
    /// let c3 = it.next_node().ok_or("missing child3")?;
    ///
    /// assert_eq!([c1.value(), c2.value(), c3.value()], ["child1", "child2", "child3"]);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn next_node(&mut self) -> Option<Node<'a, T, F>> {
        self.find(|n| n.has_children())
    }
}

impl<'a, T, F> Iterator for Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Node<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let first = self.first.take()?;
        let node = self.tree.get(first.get())?;

        if first != self.last? {
            self.first = node.next;
        }

        Some(Node::new(node, self.tree))
    }
}

impl<T, F> DoubleEndedIterator for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let last = self.last.take()?;
        let node = self.tree.get(last.get())?;

        if last != self.first? {
            self.last = node.prev;
        }

        Some(Node::new(node, self.tree))
    }
}

impl<T, F> FusedIterator for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            first: self.first,
            last: self.last,
        }
    }
}

impl<T, F> Default for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            tree: &[],
            first: None,
            last: None,
        }
    }
}
