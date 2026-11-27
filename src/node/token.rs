use core::fmt;
use core::ops::Range;

use crate::flavor::Flavor;
use crate::links::TokenLinks;
use crate::node::{Element, Node};
use crate::pointer::Pointer;
use crate::span::Span;
use crate::tree::Tree;

/// A token in the tree, one which was added with
/// [`Builder::token`][crate::Builder::token] or its variants.
///
/// Tokens have no children.
pub struct Token<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    tree: &'a Tree<T, F>,
    links: &'a TokenLinks<T, F::Index, F::Pointer>,
    id: F::Pointer,
}

impl<'a, T, F> Token<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a handle to the token with the given id. Gap entries are not
    /// tokens.
    pub(crate) fn new(tree: &'a Tree<T, F>, id: F::Pointer) -> Option<Self> {
        let links = tree.tokens().get(id.get())?;

        if links.is_gap() {
            return None;
        }

        Some(Self { tree, links, id })
    }

    /// Access the data associated with the token.
    #[must_use]
    pub fn value(&self) -> T {
        self.links.data.get()
    }

    /// Replace the value of the token with a new one, returning the old
    /// value.
    pub fn replace(&self, value: T) -> T {
        self.links.data.replace(value)
    }

    /// Get the identifier of the token, see [`Tree::token`].
    #[must_use]
    pub fn id(&self) -> F::Pointer {
        self.id
    }

    /// Get the span of the token.
    #[must_use]
    pub fn span(&self) -> Span<F::Index> {
        Span::new(self.tree.token_start(self.id.get()), self.links.end)
    }

    /// Access the [Span] of the token as a [Range].
    #[must_use]
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.span().range()
    }

    /// Get the parent node, `None` at the root of the tree.
    #[must_use]
    pub fn parent(&self) -> Option<Node<'a, T, F>> {
        Node::new(self.tree, self.links.parent?)
    }

    /// Get the previous sibling, a node or a token.
    ///
    /// This is linear in the number of interior nodes which are siblings of
    /// the token. Iterators such as [`Node::children`] step in constant time.
    #[must_use]
    pub fn prev(&self) -> Option<Element<'a, T, F>> {
        let index = self.id.get();
        let (prev, _) = self.tree.nodes_around(self.links.parent, index);
        let id = self.tree.prev_at(self.links.parent, index, prev)?;
        self.tree.element(id)
    }

    /// Get the next sibling, a node or a token.
    ///
    /// This is linear in the number of interior nodes which are siblings of
    /// the token. Iterators such as [`Node::children`] step in constant time.
    #[must_use]
    pub fn next(&self) -> Option<Element<'a, T, F>> {
        let index = self.id.get();
        let (_, next) = self.tree.nodes_around(self.links.parent, index);
        let id = self
            .tree
            .next_at(self.links.parent, index.checked_add(1)?, next)?;
        self.tree.element(id)
    }
}

impl<T, F> fmt::Debug for Token<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("data", &self.links.data.get())
            .field("span", &self.span())
            .finish()
    }
}

impl<T, F> Clone for Token<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F> Copy for Token<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, A, B> PartialEq<Token<'_, T, A>> for Token<'_, T, B>
where
    T: Copy + PartialEq,
    A: Flavor,
    B: Flavor<Index: PartialEq<A::Index>>,
{
    fn eq(&self, other: &Token<'_, T, A>) -> bool {
        self.links.data.get() == other.links.data.get() && self.span() == other.span()
    }
}

impl<T, F> Eq for Token<'_, T, F>
where
    T: Copy + Eq,
    F: Flavor<Index: Eq>,
{
}
