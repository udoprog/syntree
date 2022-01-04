use std::{fmt, mem};

use thiserror::Error;

use crate::tree::Kind;
use crate::{Id, Span, Tree};

/// A checkpoint which indicates a position in the tree where an element can be
/// optionally inserted.
#[derive(Debug, Clone, Copy)]
pub struct Checkpoint(usize);

/// Error raised by [TreeBuilder::end_node] if there currently is no node being
/// built.
///
/// # Examples
///
/// ```
/// use syntree::{EndNodeError, Span, TreeBuilder};
///
/// # fn main() -> anyhow::Result<()> {
/// let mut tree = TreeBuilder::new();
///
/// tree.start_node("root");
/// tree.end_node()?;
///
/// // Syntax::Root and Syntax::Child is left open.
/// assert!(matches!(tree.end_node(), Err(EndNodeError { .. })));
/// # Ok(()) }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("no node being built")]
pub struct EndNodeError;

/// Error raised by [TreeBuilder::build] if the tree isn't correctly
/// balanced.
///
/// # Examples
///
/// ```
/// use syntree::{BuildError, Span, TreeBuilder};
///
/// #[derive(Debug, Clone, Copy)]
/// enum Syntax {
///     Root,
///     Child,
///     Number,
/// }
///
/// # fn main() -> anyhow::Result<()> {
/// let mut tree = TreeBuilder::new();
///
/// tree.start_node(Syntax::Root);
///
/// tree.start_node(Syntax::Child);
/// tree.token(Syntax::Number, Span::new(5, 8));
/// tree.end_node()?;
///
/// tree.start_node(Syntax::Child);
///
/// // Syntax::Root and Syntax::Child is left open.
/// assert!(matches!(tree.build(), Err(BuildError { .. })));
/// # Ok(()) }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("tree is currently being built")]
pub struct BuildError;

#[derive(Debug)]
pub(crate) struct Element<T> {
    /// The data associated with the node.
    pub(crate) data: T,
    /// The kind of the element.
    pub(crate) kind: Kind,
    /// Next sibling id.
    pub(crate) next: usize,
    /// The first child element.
    pub(crate) first: usize,
}

/// A syntax tree builder.
#[derive(Debug)]
pub struct TreeBuilder<T> {
    /// Data in the tree being built.
    pub(crate) data: Vec<Element<T>>,
    /// Nodes currently being built.
    stack: Vec<usize>,
    /// The last sibling inserted.
    sibling: usize,
}

/// Build a new syntax tree.
impl<T> TreeBuilder<T> {
    /// Construct a new tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum Syntax {
    ///     Root,
    ///     Child,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node(Syntax::Root);
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    /// # Ok(()) }
    /// ```
    pub const fn new() -> Self {
        TreeBuilder {
            data: Vec::new(),
            stack: Vec::new(),
            sibling: usize::MAX,
        }
    }

    /// Start a node with the given `data`.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum Syntax {
    ///     Root,
    ///     Child,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node(Syntax::Root);
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    /// # Ok(()) }
    /// ```
    pub fn start_node(&mut self, data: T) -> Id {
        let id = self.insert(data, Kind::Node);
        self.stack.push(id);
        Id(id)
    }

    /// End a node being built. This call must be balanced with
    /// [TreeBuilder::start_node].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum Syntax {
    ///     Root,
    ///     Child,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node(Syntax::Root);
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    /// # Ok(()) }
    /// ```
    pub fn end_node(&mut self) -> Result<(), EndNodeError> {
        let head = match self.stack.pop() {
            Some(head) => head,
            None => return Err(EndNodeError),
        };

        self.sibling = head;
        Ok(())
    }

    /// Declare a token with the specified `data` and a corresponding `span`.
    ///
    /// A token is always a terminating node that has no children.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum Syntax {
    ///     Root,
    ///     Child,
    ///     Number,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node(Syntax::Root);
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.token(Syntax::Number, Span::new(5, 8));
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    /// # Ok(()) }
    /// ```
    pub fn token(&mut self, data: T, span: Span) -> Id {
        let id = self.insert(data, Kind::Token(span));
        self.sibling = id;
        Id(id)
    }

    /// Get a checkpoint corresponding to the current position in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use anyhow::Result;
    /// use syntree::{print, Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Syntax {
    ///     Root,
    ///     Number,
    ///     Lit,
    ///     Whitespace,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut b = TreeBuilder::new();
    ///
    /// let c = b.checkpoint();
    ///
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, Span::new(1, 2));
    /// b.end_node()?;
    ///
    /// b.token(Syntax::Whitespace, Span::new(2, 5));
    ///
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, Span::new(5, 7));
    /// b.token(Syntax::Lit, Span::new(7, 9));
    /// b.end_node()?;
    ///
    /// b.insert_node_at(c, Syntax::Root);
    ///
    /// let tree = b.build()?;
    ///
    /// let root = tree.first().unwrap();
    /// assert_eq!(*root.data(), Syntax::Root);
    /// assert_eq!(root.children().count(), 3);
    /// # Ok(()) }
    /// ```
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.data.len())
    }

    /// Insert a node that wraps from the given checkpointed location.
    ///
    /// # Examples
    ///
    /// ```
    /// use anyhow::Result;
    /// use syntree::{print, Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Syntax {
    ///     Root,
    ///     Number,
    ///     Lit,
    ///     Whitespace,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut b = TreeBuilder::new();
    ///
    /// let c = b.checkpoint();
    ///
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, Span::new(1, 2));
    /// b.end_node()?;
    ///
    /// b.token(Syntax::Whitespace, Span::new(2, 5));
    ///
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, Span::new(5, 7));
    /// b.token(Syntax::Lit, Span::new(7, 9));
    /// b.end_node()?;
    ///
    /// b.insert_node_at(c, Syntax::Root);
    ///
    /// let tree = b.build()?;
    ///
    /// let root = tree.first().unwrap();
    /// assert_eq!(*root.data(), Syntax::Root);
    /// assert_eq!(root.children().count(), 3);
    /// # Ok(()) }
    /// ```
    pub fn insert_node_at(&mut self, c: Checkpoint, data: T) -> Id
    where
        T: fmt::Debug,
    {
        // With the layout of this data structure this is a fairly simple
        // operation.
        let child = self.data.len();

        let removed = match self.data.get_mut(c.0) {
            Some(entry) => {
                let new = Element {
                    data,
                    kind: Kind::Node,
                    next: usize::MAX,
                    first: child,
                };

                mem::replace(entry, new)
            }
            None => {
                return Id(self.insert(data, Kind::Node));
            }
        };

        self.data.push(removed);

        // The current sibling is the newly replaced node in the tree.
        self.sibling = c.0;
        Id(c.0)
    }

    /// Construct a tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum Syntax {
    ///     Root,
    ///     Child,
    ///     Number,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node(Syntax::Root);
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.token(Syntax::Number, Span::new(5, 8));
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// # Ok(()) }
    /// ```
    ///
    /// If a tree is unbalanced during construction, building will fail with an error:
    ///
    /// ```
    /// use syntree::{BuildError, Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum Syntax {
    ///     Root,
    ///     Child,
    ///     Number,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node(Syntax::Root);
    ///
    /// tree.start_node(Syntax::Child);
    /// tree.token(Syntax::Number, Span::new(5, 8));
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Child);
    ///
    /// // Syntax::Root and Syntax::Child is left open.
    /// assert!(matches!(tree.build(), Err(BuildError { .. })));
    /// # Ok(()) }
    /// ```
    pub fn build(&self) -> Result<Tree<T>, BuildError>
    where
        T: fmt::Debug + Copy,
    {
        if !self.stack.is_empty() {
            return Err(BuildError);
        }

        Ok(crate::convert::builder_to_tree(self))
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, kind: Kind) -> usize {
        let new = self.data.len();

        let prev = std::mem::replace(&mut self.sibling, usize::MAX);
        let parent = self.stack.last().copied().unwrap_or(usize::MAX);

        self.data.push(Element {
            data,
            kind,
            next: usize::MAX,
            first: usize::MAX,
        });

        if let Some(prev) = self.data.get_mut(prev) {
            prev.next = new;
        }

        if let Some(n) = self.data.get_mut(parent) {
            if n.first == usize::MAX {
                n.first = new;
            }
        }

        new
    }
}

impl<T> Default for TreeBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
