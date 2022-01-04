use std::{fmt, mem};

use thiserror::Error;

use crate::non_max::NonMaxUsize;
use crate::tree::Kind;
use crate::{Span, Tree};

/// A checkpoint which indicates a position in the tree where an element can be
/// optionally inserted.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Checkpoint(NonMaxUsize);

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::start_node] or [TreeBuilder::token].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Id(NonMaxUsize);

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
pub(crate) struct Links<T> {
    /// The data associated with the node.
    pub(crate) data: T,
    /// The kind of the element.
    pub(crate) kind: Kind,
    /// Next sibling id.
    pub(crate) next: Option<NonMaxUsize>,
    /// The first child element.
    pub(crate) first: Option<NonMaxUsize>,
}

/// A syntax tree builder.
#[derive(Debug)]
pub struct TreeBuilder<T> {
    /// Data in the tree being built.
    data: Vec<Links<T>>,
    /// Nodes currently being built.
    stack: Vec<NonMaxUsize>,
    /// The last sibling inserted.
    sibling: Option<NonMaxUsize>,
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
            sibling: None,
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

        self.sibling = Some(head);
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
        self.sibling = Some(id);
        Id(id)
    }

    /// Get a checkpoint corresponding to the current position in the tree.
    ///
    /// # Panics
    ///
    /// This panics if the number of nodes are too many to fit in a vector on
    /// your architecture. This corresponds to [usize::max_value()].
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
        let id = NonMaxUsize::new(self.data.len()).expect("ran out of ids");
        Checkpoint(id)
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
        let child = NonMaxUsize::new(self.data.len()).expect("ran out of ids");

        let links = match self.data.get_mut(c.0.get()) {
            Some(links) => links,
            None => {
                return Id(self.insert(data, Kind::Node));
            }
        };

        let removed = mem::replace(
            links,
            Links {
                data,
                kind: Kind::Node,
                next: None,
                first: Some(child),
            },
        );

        self.data.push(removed);

        // The current sibling is the newly replaced node in the tree.
        self.sibling = Some(c.0);
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

    /// Get the links corresponding to the given id.
    pub(crate) fn get(&self, id: usize) -> Option<&Links<T>> {
        self.data.get(id)
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, kind: Kind) -> NonMaxUsize {
        let new = NonMaxUsize::new(self.data.len()).expect("ran out of ids");

        let prev = std::mem::replace(&mut self.sibling, None);
        let parent = self.stack.last().copied();

        self.data.push(Links {
            data,
            kind,
            next: None,
            first: None,
        });

        if let Some(prev) = prev.and_then(|id| self.data.get_mut(id.get())) {
            prev.next = Some(new);
        }

        if let Some(n) = parent.and_then(|id| self.data.get_mut(id.get())) {
            if n.first.is_none() {
                n.first = Some(new);
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
