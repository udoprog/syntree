use std::error::Error;
use std::fmt;
use std::mem;

use crate::non_max::NonMaxUsize;
use crate::tree::Kind;
use crate::{Span, Tree};

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::start_node] or [TreeBuilder::token].
///
/// This can be used as a checkpoint in [TreeBuilder::insert_node_at], and a
/// checkpoint can be fetched up front from [TreeBuilder::checkpoint].
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
#[derive(Debug)]
#[non_exhaustive]
pub struct EndNodeError;

impl Error for EndNodeError {}

impl fmt::Display for EndNodeError {
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
/// tree.start_node(Syntax::Number);
/// tree.token(Syntax::Lit, 3);
/// tree.end_node()?;
///
/// tree.start_node(Syntax::Number);
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
/// calls to [TreeBuilder::start_node] and [TreeBuilder::end_node].
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
/// tree.start_node(Syntax::Root);
/// tree.start_node(Syntax::Child);
/// tree.end_node()?;
/// tree.start_node(Syntax::Child);
/// tree.end_node()?;
/// tree.end_node()?;
///
/// let tree = tree.build()?;
///
/// let expected = syntree::tree! {
///     >> Syntax::Root,
///         >> Syntax::Child,
///         <<
///         >> Syntax::Child,
///         <<
///     <<
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
            cursor: 0,
        }
    }

    /// Start a node with the given `data`.
    ///
    /// This pushes a new link with the given type onto the stack which links
    /// itself onto the last sibling node that ben introduced either through
    /// [TreeBuilder::end_node] or [TreeBuilder::insert_node_at].
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

    /// End a node being built. This call must be balanced with a prior call to
    /// [TreeBuilder::start_node] and if its not will result in an
    /// [EndNodeError].
    ///
    /// This will pop a value of the stack, and set that value as the next
    /// sibling which will be used with [TreeBuilder::start_node].
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
    /// tree.start_node(Syntax::Child);
    /// tree.token(Syntax::Number, 3);
    /// tree.end_node()?;
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
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, 1);
    /// b.end_node()?;
    ///
    /// b.token(Syntax::Whitespace, 3);
    ///
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, 2);
    /// b.token(Syntax::Lit, 2);
    /// b.end_node()?;
    ///
    /// b.insert_node_at(c, Syntax::Root);
    ///
    /// let tree = b.build()?;
    ///
    /// let root = tree.first().unwrap();
    /// assert_eq!(*root.data(), Syntax::Root);
    /// assert_eq!(root.children().count(), 2);
    /// assert_eq!(root.children_with_tokens().count(), 3);
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
    /// [TreeBuilder::start_node] that is immediately closed with
    /// [TreeBuilder::end_node].
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut a = TreeBuilder::<u32>::new();
    /// a.start_node(0);
    /// let c = a.checkpoint();
    /// a.insert_node_at(c, 1);
    /// a.end_node()?;
    /// let a = a.build()?;
    ///
    /// let mut b = TreeBuilder::<u32>::new();
    /// b.start_node(0);
    /// let c = b.checkpoint();
    /// b.insert_node_at(c, 1);
    /// b.end_node()?;
    /// let b = b.build()?;
    ///
    /// assert_eq!(a, b);
    /// # Ok(()) }
    /// ```
    ///
    /// Note that this does not modify or try to balance the stack, so the last
    /// item pushed using [TreeBuilder::start_node] will still be the one popped
    /// through [TreeBuilder::end_node].
    ///
    /// # Examples
    ///
    /// ```
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
    /// b.token(Syntax::Lit, 1);
    /// b.end_node()?;
    ///
    /// b.token(Syntax::Whitespace, 3);
    ///
    /// b.start_node(Syntax::Number);
    /// b.token(Syntax::Lit, 2);
    /// b.token(Syntax::Lit, 2);
    /// b.end_node()?;
    ///
    /// b.insert_node_at(c, Syntax::Root);
    ///
    /// let tree = b.build()?;
    ///
    /// let root = tree.first().unwrap();
    /// assert_eq!(*root.data(), Syntax::Root);
    /// assert_eq!(root.children().count(), 2);
    /// assert_eq!(root.children_with_tokens().count(), 3);
    /// # Ok(()) }
    /// ```
    pub fn insert_node_at(&mut self, id: Id, data: T) -> Id {
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
    /// tree.start_node(Syntax::Child);
    /// tree.token(Syntax::Number, 3);
    /// tree.end_node()?;
    /// tree.start_node(Syntax::Child);
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     >> Syntax::Child,
    ///         (Syntax::Number, 3),
    ///     <<
    ///     == Syntax::Child,
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
    /// tree.start_node(Syntax::Number);
    /// tree.token(Syntax::Lit, 3);
    /// tree.end_node()?;
    ///
    /// tree.start_node(Syntax::Number);
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
