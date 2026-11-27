use core::fmt;
use core::ops::Range;

use crate::flavor::Flavor;
use crate::links::TokenLinks;
use crate::node::{Ancestors, Element, Event, Id, Node, NodeOrToken, Siblings, Walk, WalkEvents};
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
    ///
    /// Token ids index the token array of the tree and are separate from
    /// [node ids][Node::id]. They are not dense: an explicit span which leaves
    /// a gap after the previous token takes an internal id for the gap.
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

    /// The next interior node at the level of the token.
    ///
    /// This walks the chain of interior nodes at the level.
    fn next_node(&self) -> Option<F::Pointer> {
        self.tree.nodes_around(self.links.parent, self.id.get()).1
    }

    /// Get an iterator over this token and its ancestors, starting with the
    /// token itself and ending with its root.
    ///
    /// See [Ancestors] for documentation.
    #[must_use]
    pub fn ancestors(&self) -> Ancestors<'a, T, F> {
        Ancestors::new(Some(NodeOrToken::Token(*self)))
    }

    /// Get an iterator over this token and its following siblings, nodes and
    /// tokens.
    ///
    /// Constructing it is linear in the number of interior nodes which are
    /// siblings of the token, stepping it is constant.
    ///
    /// See [Siblings] for documentation.
    #[must_use]
    pub fn siblings(&self) -> Siblings<'a, T, F> {
        Siblings::new(
            self.tree,
            self.links.parent,
            Id::Token(self.id),
            self.next_node(),
        )
    }

    /// Walk the tree forward in depth-first order starting with this token,
    /// then continue with what follows it in the tree.
    ///
    /// Constructing it is linear in the number of interior nodes which are
    /// siblings of the token.
    ///
    /// See [Walk] for documentation.
    #[must_use]
    pub fn walk(&self) -> Walk<'a, T, F> {
        Walk::new(
            self.tree,
            Some(Id::Token(self.id)),
            self.next_node(),
            Event::Next,
        )
    }

    /// Walk from this token forwards and upwards through the tree, not
    /// including the token itself.
    ///
    /// See [Walk] for documentation.
    #[must_use]
    pub fn walk_from(&self) -> Walk<'a, T, F> {
        Walk::new(
            self.tree,
            Some(Id::Token(self.id)),
            self.next_node(),
            Event::Up,
        )
    }

    /// Walk forward from this token, emitting events indicating how the rest
    /// of the tree is being traversed.
    ///
    /// See [`WalkEvents`] for documentation.
    #[must_use]
    pub fn walk_events(&self) -> WalkEvents<'a, T, F> {
        WalkEvents::new(
            self.tree,
            Some(Id::Token(self.id)),
            self.next_node(),
            Event::Next,
        )
    }

    /// Find a preceding element which matches the given predicate.
    ///
    /// See [`Element::find_preceding`].
    pub fn find_preceding<P>(&self, predicate: P) -> Option<Element<'a, T, F>>
    where
        P: FnMut(Element<'a, T, F>) -> bool,
    {
        NodeOrToken::Token(*self).find_preceding(predicate)
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
