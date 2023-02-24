use core::iter::FusedIterator;

use crate::links::Links;
use crate::node::{Node, SkipTokens};
use crate::pointer::Pointer;
use crate::tree::Kind;

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
///             "child2" => {}
///         },
///         "child3" => {}
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.last()).map(|n| n.children()).unwrap_or_default();
/// assert_eq!(it.next().map(|n| *n.value()), None);
/// # Ok::<_, Box<dyn std::error::Error>>(())
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
///     tree.children().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
///
/// assert_eq!(
///     tree.children().rev().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root2", "root"]
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
///     root.children().rev().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["child3", "child1"]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct Children<'a, T, I, P> {
    tree: &'a [Links<T, I, P>],
    first: Option<P>,
    last: Option<P>,
}

impl<'a, T, I, P> Children<'a, T, I, P> {
    /// Construct a new child iterator.
    #[inline]
    pub(crate) const fn new(tree: &'a [Links<T, I, P>], first: Option<P>, last: Option<P>) -> Self {
        Self { tree, first, last }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this
    /// iterator. This filters out [`Kind::Token`] elements.
    ///
    /// See [`SkipTokens`] for documentation.
    #[must_use]
    pub const fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }
}

impl<T, I, P> Children<'_, T, I, P>
where
    P: Pointer,
{
    /// Get the next node from the iterator. This advances past all non-node
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     ("t1", 1),
    ///     "child1" => {},
    ///     ("t2", 1),
    ///     "child2" => {},
    ///     ("t3", 1),
    ///     "child3" => {},
    ///     ("t4", 1)
    /// };
    ///
    /// let mut it = tree.children();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(*n.value());
    /// }
    ///
    /// assert_eq!(out, ["child1", "child2", "child3"]);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn next_node(&mut self) -> Option<Node<'_, T, I, P>> {
        loop {
            let node = self.next()?;

            if matches!(node.kind(), Kind::Node) {
                return Some(node);
            }
        }
    }
}

impl<'a, T, I, P> Iterator for Children<'a, T, I, P>
where
    P: Pointer,
{
    type Item = Node<'a, T, I, P>;

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

impl<T, I, P> DoubleEndedIterator for Children<'_, T, I, P>
where
    P: Pointer,
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

impl<T, I, P> FusedIterator for Children<'_, T, I, P> where P: Pointer {}

impl<T, I, P> Clone for Children<'_, T, I, P>
where
    P: Copy,
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

impl<T, I, P> Default for Children<'_, T, I, P> {
    #[inline]
    fn default() -> Self {
        Self {
            tree: &[],
            first: None,
            last: None,
        }
    }
}
