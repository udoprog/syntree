use thiserror::Error;

use crate::tree::Kind;
use crate::{Id, Span, Tree};

/// Error raised by [TreeBuilder::end_node] if there currently is no node being
/// built.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("no node being built")]
pub struct EndNodeError;

/// Error raised by [TreeBuilder::build] if the tree isn't correctly
/// balanced.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("tree is currently being built")]
pub struct IntoTreeError;

#[derive(Debug)]
pub(crate) struct Element<T> {
    /// The data associated with the node.
    pub(crate) data: T,
    /// The kind of the element.
    pub(crate) kind: Kind,
    /// Next sibling number.
    pub(crate) next: Option<usize>,
    /// Head to the next child element.
    pub(crate) child: Option<usize>,
}

/// A syntax tree.
#[derive(Debug)]
pub struct TreeBuilder<T> {
    ///  Data in the tree being built.
    data: slab::Slab<Element<T>>,
    /// Nodes currently being built.
    stack: Vec<usize>,
    /// The last sibling inserted.
    sibling: Option<usize>,
}

/// Build a new syntax tree.
impl<T> TreeBuilder<T> {
    /// Construct a new tree.
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a node with the given `data`.
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
        Id(id.try_into().expect("identifier out of bounds"))
    }

    /// End a node being built. This call must be balanced with
    /// [TreeBuilder::start_node].
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
        Id(id.try_into().expect("identifier out of bounds"))
    }

    /// Construct a tree.
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
    ///
    /// assert!(tree.build().is_err());
    /// # Ok(()) }
    /// ```
    pub fn build(&self) -> Result<Tree<T>, IntoTreeError>
    where
        T: Copy,
    {
        if !self.stack.is_empty() {
            return Err(IntoTreeError);
        }

        Ok(crate::convert::builder_to_tree(self))
    }

    /// Get the element with the given `id`.
    pub(crate) fn get(&self, id: usize) -> Option<&Element<T>> {
        self.data.get(id)
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, kind: Kind) -> usize {
        let new = self.data.insert(Element {
            data,
            kind,
            next: None,
            child: None,
        });

        if let Some(n) = self.sibling.take().and_then(|id| self.data.get_mut(id)) {
            n.next = Some(new);
        }

        let last = self.stack.last().copied();

        if let Some(n) = last.and_then(|id| self.data.get_mut(id)) {
            if n.child.is_none() {
                n.child = Some(new);
            }
        }

        new
    }
}

impl<T> Default for TreeBuilder<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            stack: Vec::new(),
            sibling: None,
        }
    }
}
