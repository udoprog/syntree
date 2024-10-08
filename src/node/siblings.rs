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
///         "child3" => {
///             "token2"
///         }
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.next()).map(|n| n.siblings()).unwrap_or_default();
/// assert_eq!(it.next().map(|n| n.value()), None);
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// See [`Node::siblings`].
///
/// # Examples
///
/// ```
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2" => {
///                 "token1"
///             }
///         },
///         "child3" => {
///             "token2"
///         }
///     },
///     "root2" => {
///         "child4" => {
///             "token3"
///         }
///     }
/// };
///
/// let root = tree.first().ok_or("missing root")?;
///
/// assert_eq!(
///     root.siblings().map(|n| n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Siblings<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    tree: &'a [Links<T, F::Index, F::Pointer>],
    links: Option<&'a Links<T, F::Index, F::Pointer>>,
}

impl<'a, T, F> Siblings<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new child iterator.
    #[inline]
    pub(crate) const fn new(
        tree: &'a [Links<T, F::Index, F::Pointer>],
        links: &'a Links<T, F::Index, F::Pointer>,
    ) -> Self {
        Self {
            tree,
            links: Some(links),
        }
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
    /// let first = tree.first().ok_or("missing first")?;
    ///
    /// let mut it = first.siblings();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(n.value());
    /// }
    ///
    /// assert_eq!(out, ["child1", "child2", "child3"]);
    ///
    /// let mut it = first.siblings();
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

impl<'a, T, F> Iterator for Siblings<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Node<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let links = self.links.take()?;
        self.links = links.next.and_then(|id| self.tree.get(id.get()));
        Some(Node::new(links, self.tree))
    }
}

impl<T, F> FusedIterator for Siblings<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for Siblings<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            links: self.links,
        }
    }
}

impl<T, F> Default for Siblings<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            tree: &[],
            links: None,
        }
    }
}
