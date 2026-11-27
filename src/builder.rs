mod checkpoint;

use core::cell::Cell;
use core::fmt;

use crate::links::{NodeLinks, TokenLinks};
use crate::{Error, Flavor, FlavorDefault, Index, Length, Pointer, Span, Storage, Tree};

pub use self::checkpoint::Checkpoint;
use self::checkpoint::Inner;

/// A builder for a [Tree].
///
/// This maintains a stack of nodes being built which has to be balanced with
/// calls to [`Builder::open`] and [`Builder::close`].
///
/// # Type parameters and bounds
///
/// The two type parameters of the builder determine the following properties:
/// * `T` is the data stored in the tree.
/// * `F` is the [Flavor], declared with [`flavor!`][crate::flavor!]. Its
///   [`Index`][Flavor::Index] determines the numerical bounds of spans, and if
///   set to [Empty][crate::Empty] the tree does not store any spans. Its
///   [`Width`][Flavor::Width] determines the bounds of pointers in the tree,
///   which decides how many nodes can be stored in it. It also selects the
///   [`Storage`][Flavor::Storage] used and its error type.
///
/// To use the default values, use the [Builder::new][Builder::new] constructor.
///
/// # Errors
///
/// Besides the errors documented on each method, any method which adds a node
/// or token can fail with [`Error::Flavor`] if the flavor's storage fails to
/// grow.
///
/// # Examples
///
/// ```
/// let mut tree = syntree::Builder::new();
///
/// tree.open("root")?;
/// tree.open("child")?;
/// tree.close()?;
/// tree.open("child")?;
/// tree.close()?;
/// tree.close()?;
///
/// let tree = tree.build()?;
///
/// let expected = syntree::tree! {
///     "root" => {
///         "child" => {},
///         "child" => {}
///     }
/// };
///
/// assert_eq!(tree, expected);
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Builder<T, F = FlavorDefault>
where
    T: Copy,
    F: Flavor,
{
    /// Data in the tree being built.
    tree: Tree<T, F>,
    /// The state of every checkpoint handed out, indexed by [`Checkpoint`].
    checkpoints: F::Storage<Inner<F::Pointer>>,
    /// Reference the current parent to the node being built. It itself has its
    /// parent set in the tree, so that is what is used to traverse ancestors of
    /// a node.
    parent: Option<F::Pointer>,
    /// The last interior node at the current level.
    sibling: Option<F::Pointer>,
    /// The current cursor.
    cursor: F::Index,
}

impl<T> Builder<T, FlavorDefault>
where
    T: Copy,
{
    /// Construct a new tree with a default [`Span`] based on `u32`.
    ///
    /// For a constructor that can use custom bounds, use [Builder::new_with].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("root")?;
    ///
    /// tree.open("child")?;
    /// tree.token("token", 5)?;
    /// tree.close()?;
    ///
    /// tree.open("child2")?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("token", 5)
    ///         },
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self::new_with()
    }
}

impl<T, F> Builder<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new tree with a custom span.
    ///
    /// To build a tree with default bounds, see [Builder::new]. Also see the
    /// [Builder] documentation for what the different bounds means.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Builder, Empty, Tree};
    ///
    /// syntree::flavor! {
    ///     struct FlavorEmpty {
    ///         type Index = Empty;
    ///         type Width = u32;
    ///     }
    /// }
    ///
    /// let mut tree: Builder<_, FlavorEmpty> = Builder::new_with();
    ///
    /// tree.open("root")?;
    ///
    /// tree.open("child")?;
    /// tree.token("token", Empty)?;
    /// tree.close()?;
    ///
    /// tree.open("child2")?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected: Tree<_, FlavorEmpty> = syntree::tree_with! {
    ///     "root" => {
    ///         "child" => {
    ///             "token"
    ///         },
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn new_with() -> Self {
        Builder {
            tree: Tree::new_with(),
            parent: None,
            checkpoints: Storage::EMPTY,
            sibling: None,
            cursor: F::Index::EMPTY,
        }
    }

    /// Construct a new builder with capacity for `tokens` tokens.
    ///
    /// Every call to [`Builder::token`] (and its variants) adds one token,
    /// and a token with an explicit span which leaves a gap after the previous
    /// token takes one more entry. Interior nodes, added by [`Builder::open`]
    /// and [`Builder::close_at`], are sized separately through
    /// [`Builder::with_capacities`].
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Flavor`] if the flavor's storage can't hold
    /// `tokens` tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Builder, FlavorDefault};
    ///
    /// let mut tree = Builder::<_, FlavorDefault>::with_capacity(2)?;
    ///
    /// tree.open("root")?;
    /// tree.token("token", 5)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    /// assert_eq!(tree.len(), 2);
    /// assert!(tree.capacity() >= 2);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn with_capacity(tokens: usize) -> Result<Self, Error<F::Error>> {
        Self::with_capacities(0, tokens)
    }

    /// Construct a new builder with capacity for `nodes` interior nodes and
    /// `tokens` tokens.
    ///
    /// See [`Builder::with_capacity`] for what the counts mean.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Flavor`] if the flavor's storage can't hold
    /// `nodes` nodes or `tokens` tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Builder, FlavorDefault};
    ///
    /// let source = "a b";
    /// // At most one token per byte, and a root node.
    /// let mut tree = Builder::<_, FlavorDefault>::with_capacities(1, source.len())?;
    ///
    /// tree.open("root")?;
    /// tree.token("word", 1)?;
    /// tree.token("ws", 1)?;
    /// tree.token("word", 1)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    /// assert_eq!(tree.len(), 4);
    /// assert_eq!(tree.capacity(), 4);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn with_capacities(nodes: usize, tokens: usize) -> Result<Self, Error<F::Error>> {
        Ok(Builder {
            tree: Tree::with_capacity(nodes, tokens)?,
            parent: None,
            checkpoints: Storage::EMPTY,
            sibling: None,
            cursor: F::Index::EMPTY,
        })
    }

    /// Reserve capacity for at least `nodes` more interior nodes and `tokens`
    /// more tokens.
    ///
    /// See [`Builder::with_capacity`] for what the counts mean.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Flavor`] if the flavor's storage can't hold that
    /// many more nodes or tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    /// tree.reserve(1, 1)?;
    ///
    /// tree.open("root")?;
    /// tree.token("token", 5)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    /// assert_eq!(tree.len(), 2);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn reserve(&mut self, nodes: usize, tokens: usize) -> Result<(), Error<F::Error>> {
        self.tree.reserve(nodes, tokens)
    }

    /// Get a reference to the current cursor position of the syntax tree.
    ///
    /// The cursor position is the position in which it's been advanced so far
    /// as a result of calls to [Builder::token] and indicates the current
    /// starting index of the next span.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// assert_eq!(*tree.cursor(), 0);
    /// tree.open("child")?;
    /// assert_eq!(*tree.cursor(), 0);
    /// tree.token("lit", 4)?;
    /// assert_eq!(*tree.cursor(), 4);
    /// tree.close()?;
    /// assert_eq!(*tree.cursor(), 4);
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "child" => {
    ///         ("lit", 4)
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub const fn cursor(&self) -> &F::Index {
        &self.cursor
    }

    /// Set the cursor position of the syntax tree.
    ///
    /// This is used to implicitly set spans when using spanless constructions
    /// methods. Range queries such as [`Tree::node_with_range`] assume that
    /// tokens are added in non-decreasing order, so the cursor should not be
    /// moved backwards past an already added token.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("child")?;
    /// tree.token("lit", 4)?;
    /// // Skip two units of input which are not part of the tree.
    /// tree.set_cursor(6);
    /// assert_eq!(*tree.cursor(), 6);
    /// tree.token("lit", 2)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let child = tree.first().ok_or("missing child")?;
    /// assert_eq!(child.span(), &Span::new(0, 8));
    ///
    /// let spans = child.children().map(|n| n.span()).collect::<Vec<_>>();
    /// assert_eq!(spans, [Span::new(0, 4), Span::new(6, 8)]);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn set_cursor(&mut self, cursor: F::Index) {
        self.cursor = cursor;
    }

    /// Start a node with the given `data`.
    ///
    /// This pushes a new link with the given type onto the stack which links
    /// itself onto the last sibling node that was introduced either through
    /// [`Builder::close`], [`Builder::close_at`], or
    /// [`Builder::close_at_with`].
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of node
    /// identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("root")?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn open(&mut self, data: T) -> Result<F::Pointer, Error<F::Error>> {
        let id = self.insert(data, Span::point(self.cursor))?;
        self.parent = Some(id);
        Ok(id)
    }

    /// Start a node with the given `data` and span.
    ///
    /// This pushes a new link with the given type onto the stack which links
    /// itself onto the last sibling node that was introduced either through
    /// [`Builder::close`], [`Builder::close_at`] or [`Builder::close_at_with`].
    ///
    /// This does not advance the cursor position, in order to fix tree
    /// construction use [`Builder::set_cursor`].
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of node
    /// identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("root")?;
    ///
    /// tree.open_with("child", Span::new(0, 3))?;
    /// tree.close()?;
    ///
    /// tree.open_with("child", Span::new(3, 6))?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     ("root", (0, 6)) => {
    ///         ("child", (0, 3)) => {},
    ///         ("child", (3, 6)) => {}
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn open_with(
        &mut self,
        data: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let id = self.insert(data, span)?;
        self.parent = Some(id);
        Ok(id)
    }

    /// End a node being built.
    ///
    /// This will pop a value of the stack, and set that value as the next
    /// sibling which will be used with [`Builder::open`].
    ///
    /// # Errors
    ///
    /// This call must be balanced with a prior call to [`Builder::open`].
    /// If not this will result in an [`Error::CloseError`] being raised.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("root")?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn close(&mut self) -> Result<(), Error<F::Error>> {
        let head = self.parent.ok_or(Error::CloseError)?;
        let tok_end = F::Pointer::new(self.tree.tokens().len()).ok_or(Error::Overflow)?;

        let links = self
            .tree
            .node_links_mut(head)
            .ok_or_else(|| Error::MissingNode(head.get()))?;

        links.tok_end = tok_end;
        let parent = links.parent;
        let span = links.span;

        if let Some(id) = parent {
            let parent = self
                .tree
                .node_links_mut(id)
                .ok_or_else(|| Error::MissingNode(id.get()))?;

            parent.span = parent.span.join(&span);
        }

        self.parent = parent;
        self.sibling = Some(head);
        self.cursor = span.end;
        Ok(())
    }

    /// Declare a token with the specified `value` and a corresponding `len`.
    ///
    /// A token is always a terminating element without children.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of token
    /// identifiers, or [`Error::TokenOrder`] if the token would start before
    /// the end of the previous token, which happens after
    /// [`Builder::set_cursor`] moves the cursor back.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("child")?;
    /// tree.token("lit", 4)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "child" => {
    ///         ("lit", 4)
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn token(&mut self, value: T, len: F::Length) -> Result<F::Pointer, Error<F::Error>> {
        let start = self.cursor;

        let end = if len.is_empty() {
            start
        } else {
            start.checked_add_len(len).ok_or(Error::Overflow)?
        };

        self.insert_token(value, Span::new(start, end), false)
    }

    /// Insert a token with a custom span.
    ///
    /// The span is joined into the spans of the enclosing nodes and the tree.
    /// Tokens are stored in source order, so a token can't start before the
    /// end of the previous one, but it may leave a gap after it.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of token
    /// identifiers, or [`Error::TokenOrder`] if `span` starts before the end
    /// of the previous token or ends before it starts.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open_with("child", Span::new(0, 10))?;
    /// tree.token_with("lit", Span::new(2, 4))?;
    /// tree.token_with("lit2", Span::new(4, 6))?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let child = tree.first().ok_or("missing child")?;
    /// assert_eq!(child.span(), &Span::new(0, 10));
    ///
    /// let spans = child.children().map(|n| (n.value(), n.span())).collect::<Vec<_>>();
    /// assert_eq!(spans, [("lit", Span::new(2, 4)), ("lit2", Span::new(4, 6))]);
    ///
    /// assert_eq!(tree.span(), &Span::new(0, 10));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// Tokens with spans forces the cursor to be set to the end of the span.
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("child")?;
    /// tree.token_with("lit", Span::new(2, 4))?;
    /// tree.token_with("lit2", Span::new(4, 6))?;
    /// assert_eq!(*tree.cursor(), 6);
    /// tree.token("lit3", 6)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let child = tree.first().ok_or("missing child")?;
    /// assert_eq!(child.span(), &Span::new(0, 12));
    ///
    /// let lit3 = child.last().ok_or("missing lit3")?;
    /// assert_eq!(lit3.value(), "lit3");
    /// assert_eq!(lit3.span(), &Span::new(6, 12));
    ///
    /// assert_eq!(tree.span(), &Span::new(0, 12));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn token_with(
        &mut self,
        value: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        self.insert_token(value, span, false)
    }

    /// Declare a token with the specified `value` and an empty length.
    ///
    /// A token is always a terminating element without children.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of token
    /// identifiers, or [`Error::TokenOrder`] if the token would start before
    /// the end of the previous token, which happens after
    /// [`Builder::set_cursor`] moves the cursor back.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("child")?;
    /// tree.token_empty("lit")?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "child" => {
    ///         "lit"
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn token_empty(&mut self, value: T) -> Result<F::Pointer, Error<F::Error>> {
        self.token(value, F::Length::EMPTY)
    }

    /// Insert a trivia token with the given length, such as whitespace or a
    /// comment.
    ///
    /// Trivia is stored like any other token, so the spans of the enclosing
    /// nodes and of the tree cover it and the source round-trips losslessly.
    /// It is only hidden from iteration: [`Node::children`], [`Node::walk`]
    /// and the other iterators skip trivia unless asked to include it with
    /// `with_trivia()`. Point accessors such as [`Node::first`] or
    /// [`Token::next`] are structural and do see it. Use [`Token::is_trivia`]
    /// to tell it apart.
    ///
    /// [`Node::children`]: crate::Node::children
    /// [`Node::walk`]: crate::Node::walk
    /// [`Node::first`]: crate::Node::first
    /// [`Token::next`]: crate::Token::next
    /// [`Token::is_trivia`]: crate::Token::is_trivia
    ///
    /// # Errors
    ///
    /// Errors like [`Builder::token`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("call")?;
    /// tree.token("ident", 3)?;
    /// tree.trivia("whitespace", 1)?;
    /// tree.token("paren", 2)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let call = tree.first().and_then(|e| e.into_node()).ok_or("missing call")?;
    /// assert_eq!(call.span(), &syntree::Span::new(0, 6));
    ///
    /// let kinds = call.children().map(|e| e.value()).collect::<Vec<_>>();
    /// assert_eq!(kinds, ["ident", "paren"]);
    ///
    /// let kinds = call.children().with_trivia().map(|e| e.value()).collect::<Vec<_>>();
    /// assert_eq!(kinds, ["ident", "whitespace", "paren"]);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn trivia(&mut self, value: T, len: F::Length) -> Result<F::Pointer, Error<F::Error>> {
        let start = self.cursor;

        let end = if len.is_empty() {
            start
        } else {
            start.checked_add_len(len).ok_or(Error::Overflow)?
        };

        self.insert_token(value, Span::new(start, end), true)
    }

    /// Insert a trivia token with a custom span. This is [`Builder::trivia`]
    /// with an explicit span, like [`Builder::token_with`] is to
    /// [`Builder::token`].
    ///
    /// # Errors
    ///
    /// Errors like [`Builder::token_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.token_with("ident", Span::new(0, 3))?;
    /// tree.trivia_with("comment", Span::new(4, 9))?;
    ///
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.span(), &Span::new(0, 9));
    /// assert_eq!(tree.children().count(), 1);
    ///
    /// let comment = tree.last().and_then(|e| e.into_token()).ok_or("missing comment")?;
    /// assert!(comment.is_trivia());
    /// assert_eq!(comment.span(), Span::new(4, 9));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn trivia_with(
        &mut self,
        value: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        self.insert_token(value, span, true)
    }

    /// Get a checkpoint corresponding to the current position in the tree.
    ///
    /// Checkpoints are only meaningful to the builder that handed them out. A
    /// checkpoint used on another builder refers to that builder's checkpoint
    /// with the same index, or raises [`Error::MissingCheckpoint`] if it has
    /// none.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Flavor`] if the checkpoint can't be stored.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    /// tree.open("child")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    /// tree.close_at(c, "root")?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn checkpoint(&mut self) -> Result<Checkpoint<F::Pointer>, Error<F::Error>> {
        let inner = Inner {
            tok: self.tree.tokens().len(),
            before: self.sibling,
            parent: self.parent,
        };

        // Reuse the last checkpoint if it is at the same position.
        let index = self.checkpoints.len();

        if let Some(last) = index.checked_sub(1) {
            if self.checkpoints.last() == Some(&inner) {
                return Ok(Checkpoint::new(last));
            }
        }

        self.checkpoints.push(inner)?;
        Ok(Checkpoint::new(index))
    }

    /// Insert a node that wraps from the given checkpointed location.
    ///
    /// # Errors
    ///
    /// The checkpoint being closed *must* be a sibling. Otherwise a
    /// [`Error::CloseAtError`] will be raised.
    ///
    /// A checkpoint this builder did not hand out raises
    /// [`Error::MissingCheckpoint`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    /// tree.token("lit", 3)?;
    /// tree.close_at(c, "root")?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "root" => {
    ///         ("lit", 3)
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// More complex example:
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    ///
    /// tree.open("number")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    ///
    /// tree.token("whitespace", 1)?;
    ///
    /// tree.open("number")?;
    /// tree.token("lit", 2)?;
    /// tree.close()?;
    ///
    /// tree.close_at(c, "root")?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 3)
    ///         },
    ///         ("whitespace", 1),
    ///         "number" => {
    ///             ("lit", 2)
    ///         },
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// Adding a token after a checkpoint:
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
    ///
    /// let child = lit.parent().ok_or("missing child")?;
    /// assert_eq!(child.value(), "child");
    ///
    /// let root = lit.ancestors().last().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    /// assert_eq!(root.parent(), None);
    ///
    /// let expected = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     ("sibling", 3)
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn close_at(
        &mut self,
        c: Checkpoint<F::Pointer>,
        data: T,
    ) -> Result<F::Pointer, Error<F::Error>> {
        self.close_at_inner(c, data, None)
    }

    /// Insert a node with a custom length.
    ///
    /// # Errors
    ///
    /// The checkpoint being closed *must* be a sibling. Otherwise a
    /// [`Error::CloseAtError`] will be raised.
    ///
    /// A checkpoint this builder did not hand out raises
    /// [`Error::MissingCheckpoint`].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    /// tree.token_with("lit", Span::new(1, 3))?;
    /// tree.close_at_with(c, "custom", Span::new(0, 6))?;
    /// tree.token_with("lit2", Span::new(6, 8))?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "custom");
    /// assert_eq!(root.span(), &Span::new(0, 6));
    /// assert_eq!(tree.span(), &Span::new(0, 8));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn close_at_with(
        &mut self,
        c: Checkpoint<F::Pointer>,
        data: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        self.close_at_inner(c, data, Some(span))
    }

    /// Wrap everything since the checkpoint in a new node, using `span` or the
    /// span covered by the wrapped nodes when it is `None`.
    fn close_at_inner(
        &mut self,
        c: Checkpoint<F::Pointer>,
        data: T,
        span: Option<Span<F::Index>>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let Some(&Inner {
            tok,
            before,
            parent,
        }) = self.checkpoints.get(c.index())
        else {
            return Err(Error::MissingCheckpoint(c.index()));
        };

        if parent != self.parent {
            return Err(Error::CloseAtError);
        }

        let tokens = self.tree.tokens().len();

        if tok > tokens {
            return Err(Error::MissingCheckpoint(c.index()));
        }

        let new_id = self.next_node_id()?;
        let tok_start = F::Pointer::new(tok).ok_or(Error::Overflow)?;
        let tok_end = F::Pointer::new(tokens).ok_or(Error::Overflow)?;

        // The first interior node to wrap. A checkpoint whose position was
        // since wrapped by another `close_at` is stale: its interior sibling
        // has a new parent, or the interior node now at its position starts
        // before it.
        let first = match before {
            Some(id) => {
                let links = self
                    .tree
                    .node_links(id)
                    .ok_or_else(|| Error::MissingNode(id.get()))?;

                if links.parent != parent {
                    return Err(Error::CloseAtError);
                }

                links.next
            }
            None => self.tree.level_first(parent),
        };

        if let Some(id) = first {
            let links = self
                .tree
                .node_links(id)
                .ok_or_else(|| Error::MissingNode(id.get()))?;

            if links.tok_start.get() < tok {
                return Err(Error::CloseAtError);
            }
        }

        let level_last = match parent {
            Some(id) => {
                self.tree
                    .node_links(id)
                    .ok_or_else(|| Error::MissingNode(id.get()))?
                    .last
            }
            None => *self.tree.links_mut().1,
        };

        // The span of what is wrapped, from the first wrapped element to the
        // last. Nothing is wrapped if no token or interior node was added at
        // this level since the checkpoint.
        let wrapped = if first.is_some() || tok < tokens {
            let first = self.tree.next_at(parent, tok, first);
            let last = self.tree.prev_at(parent, tokens, level_last);

            match (
                first.and_then(|id| self.tree.span_of(id)),
                last.and_then(|id| self.tree.span_of(id)),
            ) {
                (Some(a), Some(b)) => Some(Span::new(a.start, b.end)),
                _ => return Err(Error::MissingNode(tok)),
            }
        } else {
            None
        };

        let node_span = match (span, wrapped) {
            (Some(span), _) => span,
            (None, Some(wrapped)) => wrapped,
            (None, None) => Span::point(self.cursor),
        };

        // Re-parent the wrapped interior nodes, and the wrapped tokens in the
        // gaps between them. Tokens inside wrapped interior nodes keep their
        // parents, so each node's token range is skipped in one step to keep
        // this linear in the number of direct children.
        let mut cur = first;
        let mut pos = tok;

        while let Some(id) = cur {
            let links = self
                .tree
                .node_links_mut(id)
                .ok_or_else(|| Error::MissingNode(id.get()))?;
            links.parent = Some(new_id);
            cur = links.next;
            let (start, end) = (links.tok_start.get(), links.tok_end.get());

            self.reparent_tokens(pos, start, parent, new_id);
            pos = pos.max(end);
        }

        self.reparent_tokens(pos, tokens, parent, new_id);

        if let Some(links) = first.and_then(|id| self.tree.node_links_mut(id)) {
            links.prev = None;
        }

        // The wrapper takes everything from the checkpoint onwards, so it is
        // always the last interior node at its level.
        match before.and_then(|id| self.tree.node_links_mut(id)) {
            Some(links) => {
                links.next = Some(new_id);
            }
            None => {
                let level = match parent {
                    Some(id) => self.tree.node_links_mut(id).map(|links| &mut links.first),
                    None => Some(self.tree.links_mut().0),
                };

                if let Some(level_first) = level {
                    *level_first = Some(new_id);
                }
            }
        }

        let level = match parent {
            Some(id) => self.tree.node_links_mut(id).map(|links| &mut links.last),
            None => Some(self.tree.links_mut().1),
        };

        if let Some(level_last) = level {
            *level_last = Some(new_id);
        }

        // An explicit span, or an empty wrapper, may reach outside of what
        // the enclosing node and the tree already cover.
        if span.is_some() || wrapped.is_none() {
            if let Some(links) = parent.and_then(|id| self.tree.node_links_mut(id)) {
                links.span = links.span.join(&node_span);
            }

            self.join_tree_span(&node_span);
        }

        self.tree.push_node(NodeLinks {
            data: Cell::new(data),
            span: node_span,
            tok_start,
            tok_end,
            parent,
            prev: before,
            next: None,
            first,
            last: first.and(level_last),
        })?;

        self.sibling = Some(new_id);
        Ok(new_id)
    }

    /// Build a [Tree] from the current state of the builder.
    ///
    /// # Errors
    ///
    /// This requires the stack in the builder to be empty. Otherwise a
    /// [`Error::BuildError`] will be raised.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("child")?;
    /// tree.token("number", 3)?;
    /// tree.close()?;
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "child" => {
    ///         ("number", 3)
    ///     },
    ///     "child" => {},
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// If a tree is unbalanced during construction, building will fail with an error:
    ///
    /// ```
    /// use syntree::{Error, Span, Builder};
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("number")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    ///
    /// tree.open("number")?;
    ///
    /// // "number" is left open.
    /// assert!(matches!(tree.build(), Err(Error::BuildError)));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn build(self) -> Result<Tree<T, F>, Error<F::Error>> {
        if self.parent.is_some() {
            return Err(Error::BuildError);
        }

        Ok(self.tree)
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, span: Span<F::Index>) -> Result<F::Pointer, Error<F::Error>> {
        let new = self.next_node_id()?;
        let tok = F::Pointer::new(self.tree.tokens().len()).ok_or(Error::Overflow)?;

        let prev = self.sibling;

        self.tree.push_node(NodeLinks {
            data: Cell::new(data),
            span,
            tok_start: tok,
            tok_end: tok,
            parent: self.parent,
            prev,
            next: None,
            first: None,
            last: None,
        })?;

        self.sibling = None;

        if let Some(id) = self.parent {
            if let Some(node) = self.tree.node_links_mut(id) {
                if node.first.is_none() {
                    node.first = Some(new);
                }

                node.last = Some(new);
                node.span = node.span.join(&span);
            }
        } else {
            let (first, last) = self.tree.links_mut();

            if first.is_none() {
                *first = Some(new);
            }

            *last = Some(new);
        }

        if let Some(node) = prev.and_then(|id| self.tree.node_links_mut(id)) {
            node.next = Some(new);
        }

        self.join_tree_span(&span);
        Ok(new)
    }

    /// Move the tokens in `start..end` which belong to `from` to `to`. An
    /// empty or out of bounds range moves nothing.
    fn reparent_tokens(
        &mut self,
        start: usize,
        end: usize,
        from: Option<F::Pointer>,
        to: F::Pointer,
    ) {
        let Some(tokens) = self.tree.tokens_mut().get_mut(start..end) else {
            return;
        };

        for links in tokens {
            if !links.is_gap() && links.parent() == from {
                links.set_parent(Some(to));
            }
        }
    }

    /// The id of the next interior node. Node ids leave the top bit of a
    /// pointer clear, since tokens store their trivia flag there.
    fn next_node_id(&self) -> Result<F::Pointer, Error<F::Error>> {
        match F::Pointer::new(self.tree.nodes().len()) {
            Some(id) if id.is_node_id() => Ok(id),
            _ => Err(Error::Overflow),
        }
    }

    /// Insert a token, preceded by a gap entry if it doesn't start where the
    /// previous token ends.
    fn insert_token(
        &mut self,
        data: T,
        span: Span<F::Index>,
        trivia: bool,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let prev_end = self.tree.tokens_end();

        if span.start < prev_end || span.end < span.start {
            return Err(Error::TokenOrder);
        }

        let gap = span.start > prev_end;

        let mut index = self.tree.tokens().len();

        if gap {
            index = index.checked_add(1).ok_or(Error::Overflow)?;
        }

        let id = F::Pointer::new(index).ok_or(Error::Overflow)?;

        // The number of tokens must itself fit in a pointer, since it is the
        // end of the token range of the enclosing nodes.
        let end = index.checked_add(1).ok_or(Error::Overflow)?;
        F::Pointer::new(end).ok_or(Error::Overflow)?;

        if gap {
            self.tree.push_token(TokenLinks::gap(data, span.start))?;
        }

        self.tree
            .push_token(TokenLinks::token(data, span.end, self.parent, trivia))?;

        if let Some(node) = self.parent.and_then(|id| self.tree.node_links_mut(id)) {
            node.span = node.span.join(&span);
        }

        self.join_tree_span(&span);
        self.cursor = span.end;
        Ok(id)
    }

    /// Extend the span of the whole tree to cover `span`. Empty spans are
    /// ignored, like they are when indexing tokens.
    fn join_tree_span(&mut self, span: &Span<F::Index>) {
        if !span.is_empty() {
            let tree = self.tree.span_mut();
            *tree = tree.join(span);
        }
    }
}

impl<T, F> Clone for Builder<T, F>
where
    T: Copy,
    F: Flavor,
    F::Storage<NodeLinks<T, F::Index, F::Pointer>>: Clone,
    F::Storage<TokenLinks<T, F::Index, F::Pointer>>: Clone,
    F::Storage<Inner<F::Pointer>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            parent: self.parent,
            checkpoints: self.checkpoints.clone(),
            sibling: self.sibling,
            cursor: self.cursor,
        }
    }
}

impl<T, F> fmt::Debug for Builder<T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("tree", &self.tree)
            .field("checkpoints", &&*self.checkpoints)
            .field("parent", &self.parent)
            .field("sibling", &self.sibling)
            .field("cursor", &self.cursor)
            .finish()
    }
}

impl<T, F> Default for Builder<T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self::new_with()
    }
}
