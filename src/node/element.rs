use core::fmt;
use core::ops::Range;

use crate::flavor::Flavor;
use crate::node::{Ancestors, Children, Node, Siblings, Token, Walk, WalkEvents};
use crate::span::Span;

/// Either a node or a token.
///
/// Iterators which visit both nodes and tokens yield the [`Element`] instance
/// of this. Its [`Debug`][fmt::Debug] output is that of the node or token it
/// holds.
#[derive(Clone, Copy, Hash)]
pub enum NodeOrToken<N, K> {
    /// An interior node.
    Node(N),
    /// A token.
    Token(K),
}

/// A node or a token in a tree.
///
/// This is what [`Tree::first`][crate::Tree::first], [`Node::first`],
/// [`Children`], [`Walk`] and the other mixed iterators yield. It has the
/// navigation methods of a [`Node`], answering as an empty leaf when it is a
/// token, so code like `tree.first()?.children()` works without first
/// checking what the element is. Use [`into_node`][NodeOrToken::into_node] or
/// [`into_token`][NodeOrToken::into_token] for what only one of them has,
/// such as [`Node::id`] or [`Token::id`].
///
/// # Examples
///
/// ```
/// let tree = syntree::tree! {
///     "root" => {
///         ("lit", 3),
///         "child" => {
///             ("ws", 1)
///         }
///     }
/// };
///
/// let root = tree.first().ok_or("missing root")?;
/// let values = root.children().map(|e| e.value()).collect::<Vec<_>>();
/// assert_eq!(values, ["lit", "child"]);
///
/// let lit = root.first().ok_or("missing lit")?;
/// assert!(lit.is_token());
/// assert_eq!(lit.children().count(), 0);
///
/// let child = lit.next().and_then(|e| e.into_node()).ok_or("missing child")?;
/// assert_eq!(tree.get(child.id()), Some(child));
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
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

    /// Replace the value of the element with a new one, returning the old
    /// value.
    pub fn replace(&self, value: T) -> T {
        match self {
            NodeOrToken::Node(n) => n.replace(value),
            NodeOrToken::Token(t) => t.replace(value),
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

    /// Test if the element is a trivia token, see [`Token::is_trivia`]. A
    /// node is never trivia.
    #[must_use]
    pub fn is_trivia(&self) -> bool {
        match self {
            NodeOrToken::Node(..) => false,
            NodeOrToken::Token(t) => t.is_trivia(),
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

    /// Get the first child, `None` for a token. See [`Node::first`].
    #[must_use]
    pub fn first(&self) -> Option<Element<'a, T, F>> {
        self.as_node()?.first()
    }

    /// Get the last child, `None` for a token. See [`Node::last`].
    #[must_use]
    pub fn last(&self) -> Option<Element<'a, T, F>> {
        self.as_node()?.last()
    }

    /// Iterate over the children of the element, which is empty for a token.
    /// See [`Node::children`].
    #[must_use]
    pub fn children(&self) -> Children<'a, T, F> {
        match self {
            NodeOrToken::Node(n) => n.children(),
            NodeOrToken::Token(..) => Children::default(),
        }
    }

    /// Get an iterator over this element and its ancestors. See
    /// [`Node::ancestors`].
    #[must_use]
    pub fn ancestors(&self) -> Ancestors<'a, T, F> {
        Ancestors::new(Some(*self))
    }

    /// Get an iterator over this element and its following siblings. See
    /// [`Node::siblings`].
    #[must_use]
    pub fn siblings(&self) -> Siblings<'a, T, F> {
        match self {
            NodeOrToken::Node(n) => n.siblings(),
            NodeOrToken::Token(t) => t.siblings(),
        }
    }

    /// Walk the tree forward in depth-first order starting with this element.
    /// See [`Node::walk`].
    #[must_use]
    pub fn walk(&self) -> Walk<'a, T, F> {
        match self {
            NodeOrToken::Node(n) => n.walk(),
            NodeOrToken::Token(t) => t.walk(),
        }
    }

    /// Walk forwards and upwards from this element, not including it or its
    /// descendants. See [`Node::walk_from`].
    #[must_use]
    pub fn walk_from(&self) -> Walk<'a, T, F> {
        match self {
            NodeOrToken::Node(n) => n.walk_from(),
            NodeOrToken::Token(t) => t.walk_from(),
        }
    }

    /// Walk forward from this element emitting events. See
    /// [`Node::walk_events`].
    #[must_use]
    pub fn walk_events(&self) -> WalkEvents<'a, T, F> {
        match self {
            NodeOrToken::Node(n) => n.walk_events(),
            NodeOrToken::Token(t) => t.walk_events(),
        }
    }

    /// Find a preceding element which matches the given predicate.
    ///
    /// A "preceding element" is one whose tokens immediately precede the
    /// tokens of the current element. This is found in two steps:
    ///
    /// 1. Starting with the current element and moving through its ancestors,
    ///    find the first [`prev`][Self::prev] sibling which matches the
    ///    predicate.
    /// 2. From that sibling, descend through [`last`][Self::last] children for
    ///    as long as each matches the predicate, and return the last match.
    ///
    /// Step 2 stops at the first last child which does not match, so a
    /// matching element nested below a non-matching last child is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child1" => {
    ///             ("token2", 1),
    ///             "child2" => {
    ///                 ("token1", 2)
    ///             }
    ///         },
    ///         "child3" => {
    ///             "child4" => {
    ///                 ("token1", 4),
    ///             }
    ///         }
    ///     }
    /// };
    ///
    /// let node = tree.node_with_range(3..3).ok_or("missing 3")?;
    /// assert_eq!(node.value(), "token1");
    ///
    /// let found = node.find_preceding(|n| n.span().end == 3 && n.has_children());
    /// let found = found.ok_or("expected preceding node")?;
    /// assert_eq!(found.value(), "child2");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn find_preceding<P>(&self, mut predicate: P) -> Option<Element<'a, T, F>>
    where
        P: FnMut(Element<'a, T, F>) -> bool,
    {
        let mut n = *self;

        let mut n = loop {
            if let Some(prev) = n.prev() {
                if predicate(prev) {
                    break prev;
                }
            }

            n = NodeOrToken::Node(n.parent()?);
        };

        loop {
            let Some(last) = n.last() else {
                return Some(n);
            };

            if !predicate(last) {
                return Some(n);
            }

            n = last;
        }
    }
}

impl<N, K> fmt::Debug for NodeOrToken<N, K>
where
    N: fmt::Debug,
    K: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrToken::Node(n) => n.fmt(f),
            NodeOrToken::Token(t) => t.fmt(f),
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
