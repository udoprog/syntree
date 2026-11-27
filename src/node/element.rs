use core::ops::Range;

use crate::flavor::Flavor;
use crate::node::{Node, Token};
use crate::span::Span;

/// Either a node or a token.
///
/// Iterators which visit both nodes and tokens yield the [`Element`] instance
/// of this.
#[derive(Debug, Clone, Copy, Hash)]
pub enum NodeOrToken<N, K> {
    /// An interior node.
    Node(N),
    /// A token.
    Token(K),
}

/// A node or a token in a tree.
pub type Element<'a, T, F> = NodeOrToken<Node<'a, T, F>, Token<'a, T, F>>;

/// The id of an element, internally.
pub(crate) type Id<P> = NodeOrToken<P, P>;

impl<N, K> NodeOrToken<N, K> {
    /// Get the node, if this is one.
    #[inline]
    pub fn into_node(self) -> Option<N> {
        match self {
            NodeOrToken::Node(n) => Some(n),
            NodeOrToken::Token(..) => None,
        }
    }

    /// Get the token, if this is one.
    #[inline]
    pub fn into_token(self) -> Option<K> {
        match self {
            NodeOrToken::Node(..) => None,
            NodeOrToken::Token(t) => Some(t),
        }
    }

    /// Get a reference to the node, if this is one.
    #[inline]
    pub const fn as_node(&self) -> Option<&N> {
        match self {
            NodeOrToken::Node(n) => Some(n),
            NodeOrToken::Token(..) => None,
        }
    }

    /// Get a reference to the token, if this is one.
    #[inline]
    pub const fn as_token(&self) -> Option<&K> {
        match self {
            NodeOrToken::Node(..) => None,
            NodeOrToken::Token(t) => Some(t),
        }
    }

    /// Test if this is a node.
    #[inline]
    pub const fn is_node(&self) -> bool {
        matches!(self, NodeOrToken::Node(..))
    }

    /// Test if this is a token.
    #[inline]
    pub const fn is_token(&self) -> bool {
        matches!(self, NodeOrToken::Token(..))
    }
}

impl<'a, T, F> NodeOrToken<Node<'a, T, F>, Token<'a, T, F>>
where
    T: Copy,
    F: Flavor,
{
    /// Access the data associated with the element.
    #[must_use]
    pub fn value(&self) -> T {
        match self {
            NodeOrToken::Node(n) => n.value(),
            NodeOrToken::Token(t) => t.value(),
        }
    }

    /// Get the span of the element.
    #[must_use]
    pub fn span(&self) -> Span<F::Index> {
        match self {
            NodeOrToken::Node(n) => *n.span(),
            NodeOrToken::Token(t) => t.span(),
        }
    }

    /// Access the [Span] of the element as a [Range].
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.span().range()
    }

    /// Get the parent node, `None` at the root of the tree.
    #[must_use]
    pub fn parent(&self) -> Option<Node<'a, T, F>> {
        match self {
            NodeOrToken::Node(n) => n.parent(),
            NodeOrToken::Token(t) => t.parent(),
        }
    }

    /// Get the previous sibling.
    #[must_use]
    pub fn prev(&self) -> Option<Element<'a, T, F>> {
        match self {
            NodeOrToken::Node(n) => n.prev(),
            NodeOrToken::Token(t) => t.prev(),
        }
    }

    /// Get the next sibling.
    #[must_use]
    pub fn next(&self) -> Option<Element<'a, T, F>> {
        match self {
            NodeOrToken::Node(n) => n.next(),
            NodeOrToken::Token(t) => t.next(),
        }
    }

    /// Check if the element has children, which a token never has.
    #[must_use]
    pub fn has_children(&self) -> bool {
        match self {
            NodeOrToken::Node(n) => n.has_children(),
            NodeOrToken::Token(..) => false,
        }
    }
}

impl<A, B, C, D> PartialEq<NodeOrToken<C, D>> for NodeOrToken<A, B>
where
    A: PartialEq<C>,
    B: PartialEq<D>,
{
    #[inline]
    fn eq(&self, other: &NodeOrToken<C, D>) -> bool {
        match (self, other) {
            (NodeOrToken::Node(a), NodeOrToken::Node(b)) => a == b,
            (NodeOrToken::Token(a), NodeOrToken::Token(b)) => a == b,
            _ => false,
        }
    }
}

impl<N, K> Eq for NodeOrToken<N, K>
where
    N: Eq,
    K: Eq,
{
}
