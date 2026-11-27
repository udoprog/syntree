use core::fmt;
use core::ops::Range;

use crate::links::{NodeLinks, TokenLinks};
use crate::node::{Children, Element, Event, Id, Node, NodeOrToken, Token, Walk, WalkEvents};
use crate::{Error, Flavor, Index, Pointer, Span, Storage};

/// A syntax tree.
///
/// A tree is constructed through a [Builder][crate::Builder] or by modifying an
/// existing tree through a [ChangeSet][crate::edit::ChangeSet].
///
/// # Type parameters and bounds
///
/// The two type parameters of the tree determine the following properties:
/// * `T` is the data stored in the tree.
/// * `F` is the [Flavor], declared with [`flavor!`][crate::flavor!]. Its
///   [`Index`][Flavor::Index] determines the numerical bounds of spans, and if
///   set to [Empty][crate::Empty] the tree does not store any spans. Its
///   [`Width`][Flavor::Width] determines the bounds of pointers in the tree,
///   which decides how many nodes can be stored in it. It also selects the
///   [`Storage`][Flavor::Storage] used and its error type.
///
/// To use the default values, use the [Builder::new][crate::Builder::new]
/// constructor.
/// # Layout
///
/// Tokens are stored in a flat array in source order, and interior nodes (the
/// ones opened with [`Builder::open`][crate::Builder::open] or wrapped with
/// [`Builder::close_at`][crate::Builder::close_at]) in a second array. Node and
/// token ids are indexes into their own array, so they are not dense: token ids
/// skip the internal entries which carry the gaps explicit spans leave between
/// tokens.
pub struct Tree<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Interior nodes.
    nodes: F::Storage<NodeLinks<T, F::Index, F::Pointer>>,
    /// Tokens and gap entries, in source order.
    tokens: F::Storage<TokenLinks<T, F::Index, F::Pointer>>,
    /// The span of the whole tree.
    span: Span<F::Index>,
    /// The first interior node at the root.
    first: Option<F::Pointer>,
    /// The last interior node at the root.
    last: Option<F::Pointer>,
    /// The number of gap entries in `tokens`.
    gaps: usize,
}

impl<T, F> Tree<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new empty tree.
    pub(crate) const fn new_with() -> Self {
        Self {
            nodes: F::Storage::EMPTY,
            tokens: F::Storage::EMPTY,
            span: Span::point(F::Index::EMPTY),
            first: None,
            last: None,
            gaps: 0,
        }
    }

    /// Construct a new tree with capacity for `nodes` interior nodes and
    /// `tokens` tokens.
    pub(crate) fn with_capacity(nodes: usize, tokens: usize) -> Result<Self, Error<F::Error>> {
        Ok(Self {
            nodes: F::Storage::with_capacity(nodes)?,
            tokens: F::Storage::with_capacity(tokens)?,
            span: Span::point(F::Index::EMPTY),
            first: None,
            last: None,
            gaps: 0,
        })
    }

    /// Reserve capacity for `nodes` more interior nodes and `tokens` more
    /// tokens.
    pub(crate) fn reserve(&mut self, nodes: usize, tokens: usize) -> Result<(), Error<F::Error>> {
        self.nodes.reserve(nodes)?;
        self.tokens.reserve(tokens)?;
        Ok(())
    }

    /// Get the span of the current node. The span of a node is the complete
    /// span of all its children.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// assert_eq!(tree.span(), Span::new(0, 13));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub const fn span(&self) -> &Span<F::Index> {
        &self.span
    }

    /// Get mutable span from the tree.
    pub(crate) fn span_mut(&mut self) -> &mut Span<F::Index> {
        &mut self.span
    }

    /// The total number of elements in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree: syntree::Builder<()> = syntree::Builder::new();
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.len(), 0);
    ///
    /// let mut tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("token", 2)
    ///         },
    ///         ("whitespace", 1),
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert_eq!(tree.len(), 5);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn len(&self) -> usize {
        self.nodes
            .len()
            .saturating_add(self.tokens.len().saturating_sub(self.gaps))
    }

    /// Check if the current tree is empty. In that it doesn't have any
    /// children at the root of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree: syntree::Builder<()> = syntree::Builder::new();
    /// let tree = tree.build()?;
    /// assert!(tree.is_empty());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.tokens.is_empty()
    }

    /// Get the capacity of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree: syntree::Builder<()> = syntree::Builder::new();
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.capacity(), 0);
    ///
    /// let mut tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("token", 2)
    ///         },
    ///         ("whitespace", 1),
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert!(tree.capacity() >= 5);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn capacity(&self) -> usize {
        self.nodes.capacity().saturating_add(self.tokens.capacity())
    }

    /// Get all root nodes in the tree.
    ///
    /// See [Children] for documentation.
    pub fn children(&self) -> Children<'_, T, F> {
        Children::new(self, None, (0, self.first), (self.tokens.len(), self.last))
    }

    /// Walk the tree forwards in a depth-first fashion visiting every node
    /// once.
    ///
    /// See [`Walk`] for documentation.
    pub fn walk(&self) -> Walk<'_, T, F> {
        Walk::tree(self, self.first_id(), self.first)
    }

    /// Walk the tree forwards in a depth-first fashion emitting events
    /// indicating how the tree is being traversed.
    ///
    /// See [`WalkEvents`] for documentation.
    pub fn walk_events(&self) -> WalkEvents<'_, T, F> {
        WalkEvents::new(self, self.first_id(), self.first, Event::Next)
    }

    /// Get the first child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {},
    ///     "root2" => {}
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn first(&self) -> Option<Element<'_, T, F>> {
        self.element(self.first_id()?)
    }

    /// Get the last child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {},
    ///     "root2" => {}
    /// };
    ///
    /// let root = tree.last().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root2");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn last(&self) -> Option<Element<'_, T, F>> {
        self.element(self.prev_at(None, self.tokens.len(), self.last)?)
    }

    /// The id of the first element in the tree.
    fn first_id(&self) -> Option<Id<F::Pointer>> {
        self.next_at(None, 0, self.first)
    }

    /// The interior nodes of the tree.
    pub(crate) fn nodes(&self) -> &[NodeLinks<T, F::Index, F::Pointer>] {
        &self.nodes
    }

    /// The token entries of the tree, gap entries included.
    pub(crate) fn tokens(&self) -> &[TokenLinks<T, F::Index, F::Pointer>] {
        &self.tokens
    }

    /// The first and last interior nodes at the root, mutably.
    pub(crate) fn links_mut(&mut self) -> (&mut Option<F::Pointer>, &mut Option<F::Pointer>) {
        (&mut self.first, &mut self.last)
    }

    /// Get an interior node.
    pub(crate) fn node_links(&self, id: F::Pointer) -> Option<&NodeLinks<T, F::Index, F::Pointer>> {
        self.nodes.get(id.get())
    }

    /// Get an interior node mutably.
    pub(crate) fn node_links_mut(
        &mut self,
        id: F::Pointer,
    ) -> Option<&mut NodeLinks<T, F::Index, F::Pointer>> {
        self.nodes.get_mut(id.get())
    }

    /// Get the token entries mutably.
    pub(crate) fn tokens_mut(&mut self) -> &mut [TokenLinks<T, F::Index, F::Pointer>] {
        &mut self.tokens
    }

    /// Push an interior node.
    pub(crate) fn push_node(
        &mut self,
        links: NodeLinks<T, F::Index, F::Pointer>,
    ) -> Result<(), F::Error> {
        self.nodes.push(links)
    }

    /// Push a token entry.
    pub(crate) fn push_token(
        &mut self,
        links: TokenLinks<T, F::Index, F::Pointer>,
    ) -> Result<(), F::Error> {
        if links.is_gap() {
            self.gaps = self.gaps.saturating_add(1);
        }

        self.tokens.push(links)
    }

    /// The start of the token entry at `index`, which is the end of the entry
    /// before it.
    pub(crate) fn token_start(&self, index: usize) -> F::Index {
        match index.checked_sub(1).and_then(|n| self.tokens.get(n)) {
            Some(prev) => prev.end,
            None => F::Index::EMPTY,
        }
    }

    /// The end of the last token entry.
    pub(crate) fn tokens_end(&self) -> F::Index {
        self.token_start(self.tokens.len())
    }

    /// The span of an element.
    pub(crate) fn span_of(&self, id: Id<F::Pointer>) -> Option<Span<F::Index>> {
        match id {
            Id::Node(id) => Some(self.node_links(id)?.span),
            Id::Token(id) => {
                let index = id.get();
                let links = self.tokens.get(index)?;
                Some(Span::new(self.token_start(index), links.end))
            }
        }
    }

    /// The element at a level which follows the position before token entry
    /// `tok`, where `node` is the next interior node at the level, if any.
    ///
    /// This is the merge step between the token array and a chain of interior
    /// nodes: an interior node comes first if its tokens start at or before
    /// `tok`.
    pub(crate) fn next_at(
        &self,
        parent: Option<F::Pointer>,
        tok: usize,
        node: Option<F::Pointer>,
    ) -> Option<Id<F::Pointer>> {
        if let Some(id) = node {
            if self.node_links(id)?.tok_start.get() <= tok {
                return Some(Id::Node(id));
            }
        }

        let mut index = tok;
        let mut links = self.tokens.get(index)?;

        // A gap entry is always directly followed by its token.
        if links.is_gap() {
            index = index.checked_add(1)?;
            links = self.tokens.get(index)?;
        }

        if links.parent != parent {
            return None;
        }

        Some(Id::Token(F::Pointer::new(index)?))
    }

    /// The element at a level which precedes the position before token entry
    /// `tok`, where `node` is the previous interior node at the level, if any.
    pub(crate) fn prev_at(
        &self,
        parent: Option<F::Pointer>,
        tok: usize,
        node: Option<F::Pointer>,
    ) -> Option<Id<F::Pointer>> {
        let mut tok = tok;

        // A position right after a gap entry is the same as the one before it,
        // since nothing is ever placed between a gap and its token.
        if let Some(index) = tok.checked_sub(1) {
            if self.tokens.get(index).is_some_and(|links| links.is_gap()) {
                tok = index;
            }
        }

        if let Some(id) = node {
            if self.node_links(id)?.tok_end.get() >= tok {
                return Some(Id::Node(id));
            }
        }

        let index = tok.checked_sub(1)?;
        let links = self.tokens.get(index)?;

        if links.is_gap() || links.parent != parent {
            return None;
        }

        Some(Id::Token(F::Pointer::new(index)?))
    }

    /// The position at a level after `id`, where `node` is the next interior
    /// node at the level if `id` is a token. This is what [`Tree::next_at`]
    /// takes to find the element which follows `id`.
    pub(crate) fn after(
        &self,
        id: Id<F::Pointer>,
        node: Option<F::Pointer>,
    ) -> Option<(usize, Option<F::Pointer>)> {
        match id {
            Id::Node(n) => {
                let links = self.node_links(n)?;
                Some((links.tok_end.get(), links.next))
            }
            Id::Token(t) => Some((t.get().checked_add(1)?, node)),
        }
    }

    /// The position at a level before `id`, where `node` is the previous
    /// interior node at the level if `id` is a token. This is what
    /// [`Tree::prev_at`] takes to find the element which precedes `id`.
    pub(crate) fn before(
        &self,
        id: Id<F::Pointer>,
        node: Option<F::Pointer>,
    ) -> Option<(usize, Option<F::Pointer>)> {
        match id {
            Id::Node(n) => {
                let links = self.node_links(n)?;
                Some((links.tok_start.get(), links.prev))
            }
            Id::Token(t) => Some((t.get(), node)),
        }
    }

    /// The first interior node at a level.
    pub(crate) fn level_first(&self, parent: Option<F::Pointer>) -> Option<F::Pointer> {
        match parent {
            Some(id) => self.node_links(id)?.first,
            None => self.first,
        }
    }

    /// The interior nodes at a level immediately before and after token entry
    /// `index`.
    ///
    /// This walks the chain of interior nodes at the level, so it is linear in
    /// their number.
    pub(crate) fn nodes_around(
        &self,
        parent: Option<F::Pointer>,
        index: usize,
    ) -> (Option<F::Pointer>, Option<F::Pointer>) {
        let mut prev = None;
        let mut cur = self.level_first(parent);

        while let Some(id) = cur {
            let Some(links) = self.node_links(id) else {
                break;
            };

            if links.tok_start.get() > index {
                break;
            }

            prev = Some(id);
            cur = links.next;
        }

        (prev, cur)
    }

    /// Construct an element handle.
    pub(crate) fn element(&self, id: Id<F::Pointer>) -> Option<Element<'_, T, F>> {
        match id {
            Id::Node(id) => Some(NodeOrToken::Node(self.get(id)?)),
            Id::Token(id) => Some(NodeOrToken::Token(self.token(id)?)),
        }
    }

    /// Get the node at the given index.
    ///
    /// Note that an id might be re-used across different trees. This behavior
    /// is never unsafe, but is not well-defined.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let node = tree.first().and_then(|n| n.last()).ok_or("missing ident")?;
    /// assert_eq!(node.value(), "ident");
    ///
    /// let id = node.into_node().ok_or("ident is a node")?.id();
    /// let node = tree.get(id).ok_or("missing ident")?;
    /// assert_eq!(node.value(), "ident");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn get(&self, id: F::Pointer) -> Option<Node<'_, T, F>> {
        Node::new(self, id)
    }

    /// Get the token with the given id.
    ///
    /// Token ids are not dense, so not every id below the number of tokens
    /// refers to a token.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    /// let id = tree.token("lit", 3)?;
    /// let tree = tree.build()?;
    ///
    /// let token = tree.token(id).ok_or("missing token")?;
    /// assert_eq!(token.value(), "lit");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn token(&self, id: F::Pointer) -> Option<Token<'_, T, F>> {
        Token::new(self, id)
    }

    /// Access the [Span] of the node as a [Range].
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// assert_eq!(tree.range(), 0..13);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.span.range()
    }

    /// Query for the smallest node which contains the given [Range].
    ///
    /// A node contains the range when it starts at or before `range.start`,
    /// ends at or after `range.end`, and `range.start` lies before its end.
    /// Tokens are nodes too, so a range within a single token returns that
    /// token, however deeply it is nested.
    ///
    /// * Returns `None` if `range.start > range.end`, or if no node contains
    ///   the range, such as one which spans two nodes at the root of the tree.
    /// * An empty range on the boundary between two tokens belongs to the token
    ///   which starts there. An empty range at the end of the tree is contained
    ///   by no node and returns `None`.
    /// * Only tokens with a non-empty span and their ancestors are considered,
    ///   so empty nodes and empty tokens are never returned.
    ///
    /// Tokens are looked up by binary search, which assumes they were added
    /// to the tree in non-decreasing order of their spans.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child1" => {
    ///             ("token1", 3)
    ///         },
    ///         "child2" => {
    ///             "nested1" => {
    ///                 ("token2", 4),
    ///             },
    ///             ("token3", 1),
    ///         },
    ///         "child3" => {
    ///             ("token4", 5)
    ///         }
    ///     },
    ///     "root2" => {}
    /// };
    ///
    /// let node = tree.node_with_range(0..0).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "token1");
    ///
    /// let node = tree.node_with_range(0..3).ok_or("missing 0..3")?;
    /// assert_eq!(node.value(), "token1");
    ///
    /// let node = tree.node_with_range(3..3).ok_or("missing 3")?;
    /// assert_eq!(node.value(), "token2");
    ///
    /// let node = tree.node_with_range(3..7).ok_or("missing 3..7")?;
    /// assert_eq!(node.value(), "token2");
    /// assert_eq!(node.parent().map(|n| n.value()), Some("nested1"));
    ///
    /// let node = tree.node_with_range(7..8).ok_or("missing 7..8")?;
    /// assert_eq!(node.value(), "token3");
    ///
    /// let node = tree.node_with_range(3..8).ok_or("missing 3..8")?;
    /// assert_eq!(node.value(), "child2");
    ///
    /// let node = tree.node_with_range(8..13).ok_or("missing 8..13")?;
    /// assert_eq!(node.value(), "token4");
    ///
    /// let node = tree.node_with_range(2..4).ok_or("missing 2..4")?;
    /// assert_eq!(node.value(), "root");
    ///
    /// // The end of the tree, where the empty "root2" sits.
    /// assert!(tree.node_with_range(13..13).is_none());
    /// assert!(tree.node_with_range(0..14).is_none());
    /// assert!(tree.node_with_range(4..2).is_none());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// Range queries work as expected with checkpoints:
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    /// tree.open("child")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    /// tree.close_at(c, "root")?;
    /// tree.token("sibling", 3)?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let lit = tree.node_with_range(0..3).ok_or("missing at 0..3")?;
    /// assert_eq!(lit.value(), "lit");
    /// assert_eq!(lit.parent().map(|n| n.value()), Some("child"));
    ///
    /// assert!(tree.node_with_range(2..4).is_none());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn node_with_range(&self, span: Range<usize>) -> Option<Element<'_, T, F>> {
        let start = F::Index::from_usize(span.start)?;
        let end = F::Index::from_usize(span.end)?;
        self.node_with_span_internal(start, end)
    }

    /// Query for the smallest node which contains the given [Span].
    ///
    /// This behaves like [Tree::node_with_range]: tokens are returned when
    /// they contain the span, and `None` is returned if `span.start >
    /// span.end`, if no node contains the span, or for an empty span at the
    /// end of the tree. Empty nodes and empty tokens are never returned.
    ///
    /// Tokens are looked up by binary search, which assumes they were added
    /// to the tree in non-decreasing order of their spans.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child1" => {
    ///             ("token1", 3)
    ///         },
    ///         "child2" => {
    ///             "nested1" => {
    ///                 ("token2", 4),
    ///             },
    ///             ("token3", 1),
    ///         },
    ///         "child3" => {
    ///             ("token4", 5)
    ///         }
    ///     },
    ///     "root2" => {}
    /// };
    ///
    /// let node = tree.node_with_span(Span::point(0)).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "token1");
    ///
    /// let node = tree.node_with_span(Span::new(0, 3)).ok_or("missing 0..3")?;
    /// assert_eq!(node.value(), "token1");
    ///
    /// let node = tree.node_with_span(Span::point(3)).ok_or("missing 3")?;
    /// assert_eq!(node.value(), "token2");
    ///
    /// let node = tree.node_with_span(Span::new(3, 7)).ok_or("missing 3..7")?;
    /// assert_eq!(node.value(), "token2");
    ///
    /// let node = tree.node_with_span(Span::new(7, 8)).ok_or("missing 7..8")?;
    /// assert_eq!(node.value(), "token3");
    ///
    /// let node = tree.node_with_span(Span::new(3, 8)).ok_or("missing 3..8")?;
    /// assert_eq!(node.value(), "child2");
    ///
    /// let node = tree.node_with_span(Span::new(8, 13)).ok_or("missing 8..13")?;
    /// assert_eq!(node.value(), "token4");
    ///
    /// let node = tree.node_with_span(Span::new(2, 4)).ok_or("missing 2..4")?;
    /// assert_eq!(node.value(), "root");
    ///
    /// assert!(tree.node_with_span(Span::point(13)).is_none());
    /// assert!(tree.node_with_span(Span::new(4, 2)).is_none());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// Range queries work as expected with checkpoints:
    ///
    /// ```
    /// use syntree::{Span, Builder};
    ///
    /// let mut tree = Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    ///
    /// tree.open("child1")?;
    /// tree.token("lit1", 3)?;
    /// tree.close()?;
    ///
    /// tree.open("child2")?;
    /// tree.token("lit2", 2)?;
    /// tree.close()?;
    ///
    /// tree.close_at(c, "root")?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let node = tree.node_with_span(Span::point(0)).ok_or("missing at point 0")?;
    /// assert_eq!(node.value(), "lit1");
    ///
    /// let node = tree.node_with_span(Span::new(0, 3)).ok_or("missing at 0..3")?;
    /// assert_eq!(node.value(), "lit1");
    /// assert_eq!(node.parent().map(|n| n.value()), Some("child1"));
    ///
    /// let node = tree.node_with_span(Span::point(3)).ok_or("missing at point 3")?;
    /// assert_eq!(node.value(), "lit2");
    ///
    /// let node = tree.node_with_span(Span::new(3, 5)).ok_or("missing at 3..5")?;
    /// assert_eq!(node.value(), "lit2");
    ///
    /// let node = tree.node_with_span(Span::new(2, 5)).ok_or("missing at 2..5")?;
    /// assert_eq!(node.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn node_with_span(&self, span: Span<F::Index>) -> Option<Element<'_, T, F>> {
        self.node_with_span_internal(span.start, span.end)
    }

    fn node_with_span_internal(&self, start: F::Index, end: F::Index) -> Option<Element<'_, T, F>> {
        if start > end {
            return None;
        }

        // Index of the first entry which ends after `start`. Empty tokens
        // never qualify, since the entry before them ends where they do.
        let n = self.tokens.partition_point(|t| t.end <= start);

        let mut next = n;

        // `start` falls in a gap left by an explicit span, so the token after
        // it starts after `start`.
        let in_gap = match self.tokens.get(n) {
            Some(links) if links.is_gap() => {
                next = n.checked_add(1)?;
                true
            }
            Some(..) => false,
            None => true,
        };

        let found = F::Pointer::new(next)
            .and_then(|id| self.element(Id::Token(id)))
            .and_then(|e| smallest_containing(e, start, end));

        if !in_gap {
            return found;
        }

        // A node spanning the gap can instead contain the closest preceding
        // token with a non-empty span.
        let mut index = n;

        let preceding = loop {
            let Some(i) = index.checked_sub(1) else {
                break None;
            };

            index = i;

            let Some(links) = self.tokens.get(i) else {
                break None;
            };

            if links.is_gap() || links.end <= self.token_start(i) {
                continue;
            }

            break F::Pointer::new(i)
                .and_then(|id| self.element(Id::Token(id)))
                .and_then(|e| smallest_containing(e, start, end));
        };

        match (found, preceding) {
            (Some(a), Some(b))
                if a.span().start <= b.span().start && b.span().end <= a.span().end =>
            {
                Some(b)
            }
            (None, b) => b,
            (a, _) => a,
        }
    }
}

/// Climb from `element` to the first element which contains `start..end`,
/// where `start` must lie strictly before the end of the element.
fn smallest_containing<'a, T, F>(
    element: Element<'a, T, F>,
    start: F::Index,
    end: F::Index,
) -> Option<Element<'a, T, F>>
where
    T: Copy,
    F: Flavor,
{
    let mut element = element;

    loop {
        let span = element.span();

        if span.start <= start && start < span.end && end <= span.end {
            return Some(element);
        }

        element = NodeOrToken::Node(element.parent()?);
    }
}

impl<T, F> Clone for Tree<T, F>
where
    T: Copy,
    F: Flavor,
    F::Storage<NodeLinks<T, F::Index, F::Pointer>>: Clone,
    F::Storage<TokenLinks<T, F::Index, F::Pointer>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            tokens: self.tokens.clone(),
            span: self.span,
            first: self.first,
            last: self.last,
            gaps: self.gaps,
        }
    }
}

impl<T, F> Default for Tree<T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self::new_with()
    }
}

impl<T, A, B> PartialEq<Tree<T, A>> for Tree<T, B>
where
    T: Copy + PartialEq,
    A: Flavor,
    B: Flavor<Index: PartialEq<A::Index>>,
{
    fn eq(&self, other: &Tree<T, A>) -> bool {
        struct Item<'a, T, F>((isize, Element<'a, T, F>))
        where
            T: Copy,
            F: Flavor;

        // NB: this is needed because the constraints on tuples doesn't allow
        // for `A` and `B` to be different.
        impl<'a, T, A, B> PartialEq<Item<'a, T, A>> for Item<'a, T, B>
        where
            T: Copy + PartialEq,
            A: Flavor,
            B: Flavor<Index: PartialEq<A::Index>>,
        {
            #[inline]
            fn eq(&self, other: &Item<'a, T, A>) -> bool {
                self.0 .0 == other.0 .0 && self.0 .1.eq(&other.0 .1)
            }
        }

        self.walk()
            .with_depths()
            .map(Item)
            .eq(other.walk().with_depths().map(Item))
    }
}

impl<T, F> Eq for Tree<T, F>
where
    T: Copy + Eq,
    F: Flavor<Index: Eq>,
{
}

impl<T, F> fmt::Debug for Tree<T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct List<'a, T, F>(&'a Tree<T, F>)
        where
            T: Copy,
            F: Flavor;

        impl<T, F> fmt::Debug for List<'_, T, F>
        where
            T: Copy + fmt::Debug,
            F: Flavor<Index: fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.walk().with_depths()).finish()
            }
        }

        f.debug_tuple("Tree").field(&List(self)).finish()
    }
}
