mod checkpoint;

use core::mem::replace;

use crate::error::Error;
use crate::index::{Index, Indexes, Length};
use crate::links::Links;
use crate::pointer::{Pointer, Width};
use crate::span::Span;
use crate::tree::Tree;

pub use self::checkpoint::Checkpoint;

/// A builder for a [Tree].
///
/// This maintains a stack of nodes being built which has to be balanced with
/// calls to [`Builder::open`] and [`Builder::close`].
///
/// # Type parameters and bounds
///
/// The three type parameters of the tree determines the following properties:
/// * `T` is the data stored in the tree.
/// * `I` determines the numerical bounds of spans stored in the tree through
///   the [Index] trait, if set to [Empty][crate::Empty] the tree does not store
///   any spans.
/// * `W` determines the bounds of pointers in the tree through the [Width]
///   trait, this decides how many elements that can be stored in the tree.
///
/// To use the default values, use the [Builder::new][Builder::new] constructor.
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
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct Builder<T, I, W>
where
    I: Index,
    W: Width,
{
    /// Data in the tree being built.
    tree: Tree<T, I, W>,
    /// The last checkpoint that was handed out.
    checkpoint: Option<Checkpoint<W::Pointer>>,
    /// Reference the current parent to the node being built.
    /// It itself has its parent set in the tree, so that is what is used to traverse ancestors of a node.
    parent: Option<W::Pointer>,
    /// Reference to last sibling inserted.
    sibling: Option<W::Pointer>,
    /// The current cursor.
    cursor: I,
}

impl<T> Builder<T, u32, usize> {
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self::new_with()
    }
}

impl<T, I, W> Builder<T, I, W>
where
    I: Index,
    W: Width,
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
    /// let mut tree: Builder<_, Empty, usize> = Builder::new_with();
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
    /// let expected: Tree<_, Empty, u32> = syntree::tree_with! {
    ///     "root" => {
    ///         "child" => {
    ///             "token"
    ///         },
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn new_with() -> Self {
        Builder {
            tree: Tree::new_with(),
            parent: None,
            checkpoint: None,
            sibling: None,
            cursor: I::EMPTY,
        }
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub const fn cursor(&self) -> &I {
        &self.cursor
    }

    /// Start a node with the given `data`.
    ///
    /// This pushes a new link with the given type onto the stack which links
    /// itself onto the last sibling node that ben introduced either through
    /// [`Builder::close`] or [`Builder::close_at`].
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn open(&mut self, data: T) -> Result<W::Pointer, Error> {
        let id = self.insert(data, Span::point(self.cursor))?;
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn close(&mut self) -> Result<(), Error> {
        let head = self.parent.take().ok_or(Error::CloseError)?;
        self.sibling = Some(head);

        let node = self
            .tree
            .get_mut(head)
            .ok_or_else(|| Error::MissingNode(head.get()))?;

        if let Some(parent) = node.parent {
            self.tree
                .get_mut(parent)
                .ok_or_else(|| Error::MissingNode(parent.get()))?
                .span
                .end = node.span.end;

            self.parent = Some(parent);
        }

        Ok(())
    }

    /// Declare a token with the specified `value` and a corresponding `len`.
    ///
    /// A token is always a terminating element without children.
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn token(&mut self, value: T, len: I::Length) -> Result<W::Pointer, Error> {
        let start = self.cursor;

        if !len.is_empty() {
            self.cursor = self.cursor.checked_add_len(len).ok_or(Error::Overflow)?;
            self.tree.span_mut().end = self.cursor;
        }

        let id = self.insert(value, Span::new(start, self.cursor))?;
        self.sibling = Some(id);

        if !len.is_empty() {
            self.tree.indexes_mut().push(self.cursor, id);
        }

        Ok(id)
    }

    /// Declare a token with the specified `value` and an empty length.
    ///
    /// A token is always a terminating element without children.
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn token_empty(&mut self, value: T) -> Result<W::Pointer, Error> {
        self.token(value, I::Length::EMPTY)
    }

    /// Get a checkpoint corresponding to the current position in the tree.
    ///
    /// # Mixing checkpoints
    ///
    /// Note that using checkpoints from a different tree doesn't have a
    /// well-specified behavior - it might seemingly work or it might raise an
    /// error during closing such as [`Error::MissingNode`].
    ///
    /// The following *is not* well-defined, here we're using a checkpoint for a
    /// different tree but it "just happens" to work because both trees have
    /// identical internal topologies:
    ///
    /// ```
    /// use syntree::{Builder, Error};
    ///
    /// let mut a = Builder::new();
    /// let mut b = Builder::new();
    ///
    /// let c = b.checkpoint()?;
    ///
    /// a.open("child")?;
    /// a.close()?;
    ///
    /// b.open("child")?;
    /// b.close()?;
    ///
    /// // Checkpoint use from different tree.
    /// a.close_at(&c, "root")?;
    ///
    /// let unexpected = syntree::tree! {
    ///     "root" => {
    ///         "child"
    ///     }
    /// };
    ///
    /// assert_eq!(a.build()?, unexpected);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of node
    /// identifiers.
    ///
    /// # Panics
    ///
    /// This panics if the number of nodes are too many to fit in a vector on
    /// your architecture. This corresponds to [`usize::MAX`].
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
    /// tree.close_at(&c, "root")?;
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn checkpoint(&mut self) -> Result<Checkpoint<W::Pointer>, Error> {
        let node = W::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        if let Some(c) = &self.checkpoint {
            if c.node() == node {
                return Ok(c.clone());
            }
        }

        let c = Checkpoint::new(node, self.parent);
        self.checkpoint = Some(c.clone());
        Ok(c)
    }

    /// Insert a node that wraps from the given checkpointed location.
    ///
    /// # Errors
    ///
    /// The checkpoint being closed *must* be a sibling. Otherwise a
    /// [`Error::CloseAtError`] will be raised.
    ///
    /// This might also sporadically error with [`Error::MissingNode`], in case
    /// a checkpoint is used that was constructed from another tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    /// tree.token("lit", 3)?;
    /// tree.close_at(&c, "root")?;
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
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
    /// tree.close_at(&c, "root")?;
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
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
    /// tree.close_at(&c, "root")?;
    /// tree.token("sibling", 3)?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let child = tree.node_with_range(0..3).ok_or("missing at 0..3")?;
    /// assert_eq!(*child.value(), "child");
    ///
    /// let lit = tree.first().and_then(|n| n.first()).and_then(|n| n.first()).ok_or("expected lit")?;
    /// assert_eq!(*lit.value(), "lit");
    ///
    /// let root = lit.ancestors().last().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root");
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn close_at(&mut self, c: &Checkpoint<W::Pointer>, data: T) -> Result<W::Pointer, Error> {
        let (id, parent) = c.get();

        if parent != self.parent {
            return Err(Error::CloseAtError);
        }

        let new_id = W::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        let Some(links) = self.tree.get_mut(id) else {
            let new_id = self.insert(data, Span::point(self.cursor))?;

            if new_id != id {
                return Err(Error::MissingNode(new_id.get()));
            }

            self.sibling = Some(new_id);
            return Ok(new_id);
        };

        let parent = replace(&mut links.parent, Some(new_id));
        let prev = replace(&mut links.prev, None);

        // Restructuring is necessary to calculate the full span of the newly
        // inserted node and update parent references to point to the newly
        // inserted node.
        let (last, span) = if let Some(next) = links.next {
            let span = links.span;
            let (last, end) = restructure_close_at(&mut self.tree, new_id, next)?;
            (last, Span::new(span.start, end))
        } else {
            (id, links.span)
        };

        if let Some(parent) = parent.and_then(|id| self.tree.get_mut(id)) {
            if parent.first == Some(id) {
                parent.first = Some(new_id);
            }

            if parent.last == Some(id) {
                parent.last = Some(new_id);
            }
        }

        if let Some(prev) = prev.and_then(|id| self.tree.get_mut(id)) {
            prev.next = Some(new_id);
        }

        // If we're replacing the first node of the tree, the newly inserted
        // node should be set as the first node.
        let (first, _) = self.tree.links_mut();

        if *first == Some(id) {
            *first = Some(new_id);
        }

        // Do necessary accounting.
        self.tree.push(Links {
            data,
            span,
            prev,
            parent,
            next: None,
            first: Some(id),
            last: Some(last),
        });

        self.sibling = Some(new_id);
        c.set(new_id, parent);
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn build(self) -> Result<Tree<T, I, W>, Error> {
        if self.parent.is_some() {
            return Err(Error::BuildError);
        }

        Ok(self.tree)
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, span: Span<I>) -> Result<W::Pointer, Error> {
        let new = W::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        let prev = self.sibling.take();

        self.tree.push(Links {
            data,
            span,
            parent: self.parent,
            prev,
            next: None,
            first: None,
            last: None,
        });

        if let Some(id) = self.parent {
            if let Some(node) = self.tree.links_at_mut(id) {
                if node.first.is_none() {
                    node.first = Some(new);
                }

                node.last = Some(new);
                node.span.end = span.end;
            }
        } else {
            let (first, last) = self.tree.links_mut();

            if first.is_none() {
                *first = Some(new);
            }

            *last = Some(new);
        }

        if let Some(node) = prev.and_then(|id| self.tree.links_at_mut(id)) {
            node.next = Some(new);
        }

        Ok(new)
    }
}

impl<T, I, W> Clone for Builder<T, I, W>
where
    T: Clone,
    I: Index,
    I::Indexes<W::Pointer>: Clone,
    W: Width,
    W::Pointer: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            parent: self.parent,
            checkpoint: self.checkpoint.clone(),
            sibling: self.sibling,
            cursor: self.cursor,
        }
    }
}

impl<T, I, W> Default for Builder<T, I, W>
where
    I: Index,
    W: Width,
{
    #[inline]
    fn default() -> Self {
        Self::new_with()
    }
}

// Adjust span to encapsulate all children and check that we just inserted the
// checkpointed node in the right location which should be the tail sibling of
// the replaced node.
fn restructure_close_at<T, I, W>(
    tree: &mut Tree<T, I, W>,
    parent_id: W::Pointer,
    next: W::Pointer,
) -> Result<(W::Pointer, I), Error>
where
    I: Index,
    W: Width,
{
    let mut links = tree
        .get_mut(next)
        .ok_or_else(|| Error::MissingNode(next.get()))?;
    let mut last = (next, links.span.end);
    links.parent = Some(parent_id);

    while let Some(next) = links.next {
        links = tree
            .get_mut(next)
            .ok_or_else(|| Error::MissingNode(next.get()))?;
        last = (next, links.span.end);
        links.parent = Some(parent_id);
    }

    Ok(last)
}
