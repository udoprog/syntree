use core::fmt;
use core::ops::Range;

use crate::flavor::Flavor;
use crate::links::NodeLinks;
use crate::node::{Children, Element, Id, Walk, WalkEvents};
use crate::pointer::Pointer;
use crate::span::Span;
use crate::tree::Tree;

/// An interior node in the tree, one which was opened with
/// [`Builder::open`][crate::Builder::open] or wrapped with
/// [`Builder::close_at`][crate::Builder::close_at].
///
/// Tokens are [`Token`][crate::node::Token]s, and an iterator which mixes both
/// yields [`Element`]s.
///
/// # Type parameters and bounds
///
/// The two type parameters of the tree determine the following properties:
/// * `T` is the data stored in the tree.
/// * `F` determines the [Flavor] of the tree, defining numerical bounds of
///   spans and node identifiers stored in the tree.
///
/// A tree built through [Builder::new][crate::Builder::new] uses
/// [`FlavorDefault`][crate::FlavorDefault].
pub struct Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    tree: &'a Tree<T, F>,
    links: &'a NodeLinks<T, F::Index, F::Pointer>,
    id: F::Pointer,
}

impl<'a, T, F> Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a handle to the node with the given id.
    pub(crate) fn new(tree: &'a Tree<T, F>, id: F::Pointer) -> Option<Self> {
        let links = tree.node_links(id)?;
        Some(Self { tree, links, id })
    }

    /// Access the data associated with the node.
    #[must_use]
    pub fn value(&self) -> T {
        self.links.data.get()
    }

    /// Replace the value of the node with a new one, returning the old value.
    pub fn replace(&self, value: T) -> T {
        self.links.data.replace(value)
    }

    /// Get the identifier of the node, see [`Tree::get`].
    #[must_use]
    pub fn id(&self) -> F::Pointer {
        self.id
    }

    /// Get the span of the node.
    #[must_use]
    pub const fn span(&self) -> &Span<F::Index> {
        &self.links.span
    }

    /// Access the [Span] of the node as a [Range].
    #[must_use]
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.links.span.range()
    }

    /// Check if the node has children, tokens or nodes.
    #[must_use]
    pub fn has_children(&self) -> bool {
        self.first().is_some()
    }

    /// Check if the node is empty, in that it has no children.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.has_children()
    }

    /// Get the parent node, `None` at the root of the tree.
    #[must_use]
    pub fn parent(&self) -> Option<Node<'a, T, F>> {
        Node::new(self.tree, self.links.parent?)
    }

    /// Get the previous sibling, a node or a token.
    #[must_use]
    pub fn prev(&self) -> Option<Element<'a, T, F>> {
        let id = self.tree.prev_at(
            self.links.parent,
            self.links.tok_start.get(),
            self.links.prev,
        )?;
        self.tree.element(id)
    }

    /// Get the next sibling, a node or a token.
    #[must_use]
    pub fn next(&self) -> Option<Element<'a, T, F>> {
        let id = self
            .tree
            .next_at(self.links.parent, self.links.tok_end.get(), self.links.next)?;
        self.tree.element(id)
    }

    /// Get the first child, a node or a token.
    #[must_use]
    pub fn first(&self) -> Option<Element<'a, T, F>> {
        let id = self
            .tree
            .next_at(Some(self.id), self.links.tok_start.get(), self.links.first)?;
        self.tree.element(id)
    }

    /// Get the last child, a node or a token.
    #[must_use]
    pub fn last(&self) -> Option<Element<'a, T, F>> {
        let id = self
            .tree
            .prev_at(Some(self.id), self.links.tok_end.get(), self.links.last)?;
        self.tree.element(id)
    }

    /// Iterate over the children of the node, nodes and tokens in order.
    #[must_use]
    pub fn children(&self) -> Children<'a, T, F> {
        Children::new(
            self.tree,
            Some(self.id),
            (self.links.tok_start.get(), self.links.first),
            (self.links.tok_end.get(), self.links.last),
        )
    }

    /// Walk the subtree forward starting with this node, then continue with
    /// what follows it in the tree.
    #[must_use]
    pub fn walk(&self) -> Walk<'a, T, F> {
        Walk::new(self.tree, Some(Id::Node(self.id)), None)
    }

    /// Walk the subtree forward starting with this node, emitting events
    /// which indicate how the tree is being traversed.
    #[must_use]
    pub fn walk_events(&self) -> WalkEvents<'a, T, F> {
        WalkEvents::new(self.tree, Some(Id::Node(self.id)), None)
    }
}

impl<T, F> fmt::Debug for Node<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.links.data.get())
            .field("span", &self.links.span)
            .finish()
    }
}

impl<T, F> Clone for Node<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F> Copy for Node<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, A, B> PartialEq<Node<'_, T, A>> for Node<'_, T, B>
where
    T: Copy + PartialEq,
    A: Flavor,
    B: Flavor<Index: PartialEq<A::Index>>,
{
    fn eq(&self, other: &Node<'_, T, A>) -> bool {
        self.links.data.get() == other.links.data.get() && self.links.span == other.links.span
    }
}

impl<T, F> Eq for Node<'_, T, F>
where
    T: Copy + Eq,
    F: Flavor<Index: Eq>,
{
}
