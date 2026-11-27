use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::node::{Element, Id, Node, NodeOrToken, SkipTokens};
use crate::tree::Tree;

/// An iterator over the children of a node or the root of a tree, nodes and
/// tokens in order.
///
/// The children of a level are its tokens merged in source order with its
/// chain of interior nodes, so both directions step in constant time.
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
/// See [`Tree::children`][crate::Tree::children] or [`Node::children`][crate::Node::children].
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
    tree: Option<&'a Tree<T, F>>,
    /// The node whose children are iterated, `None` at the root.
    parent: Option<F::Pointer>,
    /// The token position and next interior node at the front.
    front: (usize, Option<F::Pointer>),
    /// The token position and previous interior node at the back.
    back: (usize, Option<F::Pointer>),
    /// The last element yielded from the front.
    front_last: Option<Id<F::Pointer>>,
    /// The last element yielded from the back.
    back_last: Option<Id<F::Pointer>>,
    done: bool,
}

impl<'a, T, F> Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    pub(crate) fn new(
        tree: &'a Tree<T, F>,
        parent: Option<F::Pointer>,
        front: (usize, Option<F::Pointer>),
        back: (usize, Option<F::Pointer>),
    ) -> Self {
        Self {
            tree: Some(tree),
            parent,
            front,
            back,
            front_last: None,
            back_last: None,
            done: false,
        }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this iterator.
    /// This filters out tokens and yields only interior [`Node`]s.
    ///
    /// See [`SkipTokens`] for documentation.
    #[must_use]
    pub const fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }

    /// Get the next interior node from the iterator, advancing past tokens.
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
        self.find_map(NodeOrToken::into_node)
    }

    /// Peek the next element from the front.
    fn peek_front(&self) -> Option<Id<F::Pointer>> {
        if self.done {
            return None;
        }

        let id = self
            .tree?
            .next_at(self.parent, self.front.0, self.front.1)?;

        if Some(id) == self.back_last {
            return None;
        }

        Some(id)
    }

    /// Peek the next element from the back.
    fn peek_back(&self) -> Option<Id<F::Pointer>> {
        if self.done {
            return None;
        }

        let id = self.tree?.prev_at(self.parent, self.back.0, self.back.1)?;

        if Some(id) == self.front_last {
            return None;
        }

        Some(id)
    }
}

impl<'a, T, F> Iterator for Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Element<'a, T, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.tree?;

        let Some(id) = self.peek_front() else {
            self.done = true;
            return None;
        };

        self.front = tree.after(id, self.front.1)?;
        self.front_last = Some(id);
        tree.element(id)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }

        let len = self.tree.map_or(0, Tree::len);
        (usize::from(self.peek_front().is_some()), Some(len))
    }
}

impl<T, F> DoubleEndedIterator for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let tree = self.tree?;

        let Some(id) = self.peek_back() else {
            self.done = true;
            return None;
        };

        self.back = tree.before(id, self.back.1)?;
        self.back_last = Some(id);
        tree.element(id)
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
            parent: self.parent,
            front: self.front,
            back: self.back,
            front_last: self.front_last,
            back_last: self.back_last,
            done: self.done,
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
            tree: None,
            parent: None,
            front: (0, None),
            back: (0, None),
            front_last: None,
            back_last: None,
            done: true,
        }
    }
}

impl<T, F> fmt::Debug for Children<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let element = |id| self.tree.and_then(|tree| tree.element(id));

        f.debug_struct("Children")
            .field("first", &self.peek_front().and_then(element))
            .field("last", &self.peek_back().and_then(element))
            .finish()
    }
}
