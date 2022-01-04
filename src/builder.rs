use std::error::Error;
use std::fmt;
use std::mem;

use crate::non_max::NonMaxUsize;
use crate::tree::Kind;
use crate::{Span, Tree};

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::open] or [TreeBuilder::token].
///
/// This can be used as a checkpoint in [TreeBuilder::close_at], and a
/// checkpoint can be fetched up front from [TreeBuilder::checkpoint].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Id(NonMaxUsize);

/// Error raised by [TreeBuilder::close] if there currently is no node being
/// built.
///
/// # Examples
///
/// ```
/// use syntree::{CloseError, Span, TreeBuilder};
///
/// # fn main() -> anyhow::Result<()> {
/// let mut tree = TreeBuilder::new();
///
/// tree.open("root");
/// tree.close()?;
///
/// // Syntax::Root and Syntax::Child is left open.
/// assert!(matches!(tree.close(), Err(CloseError { .. })));
/// # Ok(()) }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct CloseError;

impl Error for CloseError {}

impl fmt::Display for CloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no node being built")
    }
}

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
///     Number,
///     Lit,
/// }
///
/// # fn main() -> anyhow::Result<()> {
/// let mut tree = TreeBuilder::new();
///
/// tree.open(Syntax::Number);
/// tree.token(Syntax::Lit, 3);
/// tree.close()?;
///
/// tree.open(Syntax::Number);
///
/// // Syntax::Number is left open.
/// assert!(matches!(tree.build(), Err(BuildError { .. })));
/// # Ok(()) }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct BuildError;

impl Error for BuildError {}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tree is currently being built")
    }
}

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

/// A builder for a [Tree].
///
/// This maintains a stack of nodes being built which has to be balanced with
/// calls to [TreeBuilder::open] and [TreeBuilder::close].
///
/// # Examples
///
/// ```
/// use syntree::TreeBuilder;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Syntax {
///     Root,
///     Child,
/// }
///
/// # fn main() -> anyhow::Result<()> {
/// let mut tree = TreeBuilder::new();
///
/// tree.open(Syntax::Root);
/// tree.open(Syntax::Child);
/// tree.close()?;
/// tree.open(Syntax::Child);
/// tree.close()?;
/// tree.close()?;
///
/// let tree = tree.build()?;
///
/// let expected = syntree::tree! {
///     Syntax::Root => {
///         Syntax::Child,
///         Syntax::Child,
///     }
/// };
///
/// assert_eq!(tree, expected);
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct TreeBuilder<T> {
    /// Data in the tree being built.
    data: Vec<Links<T>>,
    /// Nodes currently being built.
    stack: Vec<NonMaxUsize>,
    /// The last sibling inserted.
    sibling: Option<NonMaxUsize>,
    /// The current cursor to the source code being referenced.
    cursor: usize,
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
    /// tree.open(Syntax::Root);
    ///
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub const fn new() -> Self {
        TreeBuilder {
            data: Vec::new(),
            stack: Vec::new(),
            sibling: None,
            cursor: 0,
        }
    }

    /// Start a node with the given `data`.
    ///
    /// This pushes a new link with the given type onto the stack which links
    /// itself onto the last sibling node that ben introduced either through
    /// [TreeBuilder::close] or [TreeBuilder::close_at].
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
    /// tree.open(Syntax::Root);
    ///
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub fn open(&mut self, data: T) -> Id {
        let id = self.insert(data, Kind::Node);
        self.stack.push(id);
        Id(id)
    }

    /// End a node being built. This call must be balanced with a prior call to
    /// [TreeBuilder::open] and if its not will result in an
    /// [CloseError].
    ///
    /// This will pop a value of the stack, and set that value as the next
    /// sibling which will be used with [TreeBuilder::open].
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
    /// tree.open(Syntax::Root);
    ///
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub fn close(&mut self) -> Result<(), CloseError> {
        let head = match self.stack.pop() {
            Some(head) => head,
            None => return Err(CloseError),
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
    /// tree.open(Syntax::Child);
    /// tree.token(Syntax::Number, 3);
    /// tree.close()?;
    ///
    /// # Ok(()) }
    /// ```
    pub fn token(&mut self, data: T, len: usize) -> Id {
        let start = self.cursor;
        self.cursor = self.cursor.checked_add(len).expect("length overflow");
        let id = self.insert(data, Kind::Token(Span::new(start, self.cursor)));
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
    /// b.open(Syntax::Number);
    /// b.token(Syntax::Lit, 1);
    /// b.close()?;
    ///
    /// b.token(Syntax::Whitespace, 3);
    ///
    /// b.open(Syntax::Number);
    /// b.token(Syntax::Lit, 2);
    /// b.token(Syntax::Lit, 2);
    /// b.close()?;
    ///
    /// b.close_at(c, Syntax::Root);
    ///
    /// let tree = b.build()?;
    ///
    /// let root = tree.first().unwrap();
    /// assert_eq!(*root.data(), Syntax::Root);
    /// assert_eq!(root.children().count(), 3);
    /// assert_eq!(root.children().without_tokens().count(), 2);
    /// # Ok(()) }
    /// ```
    pub fn checkpoint(&self) -> Id {
        let id = NonMaxUsize::new(self.data.len()).expect("ran out of ids");
        Id(id)
    }

    /// Insert a node that wraps from the given checkpointed location.
    ///
    /// This causes the node specified at `id` to become the previous sibling
    /// node. If `id` refers to a node that hasn't been allocated yet (through
    /// [TreeBuilder::checkpoint]), this call corresponds exactly to
    /// [TreeBuilder::open] that is immediately closed with
    /// [TreeBuilder::close].
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut a = TreeBuilder::<u32>::new();
    /// a.open(0);
    /// let c = a.checkpoint();
    /// a.close_at(c, 1);
    /// a.close()?;
    /// let a = a.build()?;
    ///
    /// let mut b = TreeBuilder::<u32>::new();
    /// b.open(0);
    /// let c = b.checkpoint();
    /// b.close_at(c, 1);
    /// b.close()?;
    /// let b = b.build()?;
    ///
    /// assert_eq!(a, b);
    /// # Ok(()) }
    /// ```
    ///
    /// Note that this does not modify or try to balance the stack, so the last
    /// item pushed using [TreeBuilder::open] will still be the one popped
    /// through [TreeBuilder::close].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{print, Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Syntax {
    ///     ROOT,
    ///     NUMBER,
    ///     LIT,
    ///     WHITESPACE,
    /// }
    ///
    /// use Syntax::*;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut b = TreeBuilder::new();
    ///
    /// let c = b.checkpoint();
    ///
    /// b.open(NUMBER);
    /// b.token(LIT, 3);
    /// b.close()?;
    ///
    /// b.token(WHITESPACE, 1);
    ///
    /// b.open(NUMBER);
    /// b.token(LIT, 2);
    /// b.close()?;
    ///
    /// b.close_at(c, ROOT);
    ///
    /// let tree = b.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     ROOT => {
    ///         NUMBER => {
    ///             (LIT, 3)
    ///         },
    ///         (WHITESPACE, 1),
    ///         NUMBER => {
    ///             (LIT, 2)
    ///         },
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok(()) }
    /// ```
    pub fn close_at(&mut self, id: Id, data: T) -> Id {
        // With the layout of this data structure this is a fairly simple
        // operation.
        let child = NonMaxUsize::new(self.data.len()).expect("ran out of ids");

        let links = match self.data.get_mut(id.0.get()) {
            Some(links) => links,
            None => {
                let id = self.insert(data, Kind::Node);
                self.sibling = Some(id);
                return Id(id);
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
        self.sibling = Some(id.0);
        Id(id.0)
    }

    /// Build a [Tree] from the current state of the builder.
    ///
    /// This requires the stack in the builder to be empty. Otherwise a
    /// [BuildError] will be raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// enum Syntax {
    ///     Child,
    ///     Number,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open(Syntax::Child);
    /// tree.token(Syntax::Number, 3);
    /// tree.close()?;
    /// tree.open(Syntax::Child);
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     Syntax::Child => {
    ///         (Syntax::Number, 3)
    ///     },
    ///     Syntax::Child,
    /// };
    ///
    /// assert_eq!(tree, expected);
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
    ///     Number,
    ///     Lit,
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open(Syntax::Number);
    /// tree.token(Syntax::Lit, 3);
    /// tree.close()?;
    ///
    /// tree.open(Syntax::Number);
    ///
    /// // Syntax::Number is left open.
    /// assert!(matches!(tree.build(), Err(BuildError { .. })));
    /// # Ok(()) }
    /// ```
    pub fn build(&self) -> Result<Tree<T>, BuildError>
    where
        T: Clone,
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
