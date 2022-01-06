use std::mem;

use crate::links::Links;
use crate::non_max::NonMaxUsize;
use crate::{Kind, Span, Tree, TreeError};

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::open] or [TreeBuilder::token].
///
/// This can be used as a checkpoint in [TreeBuilder::close_at], and a
/// checkpoint can be fetched up front from [TreeBuilder::checkpoint].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(NonMaxUsize);

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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut tree = TreeBuilder::new();
///
/// tree.open("root");
/// tree.open("child");
/// tree.close()?;
/// tree.open("child");
/// tree.close()?;
/// tree.close()?;
///
/// let tree = tree.build()?;
///
/// let expected = syntree::tree! {
///     "root" => {
///         "child",
///         "child"
///     }
/// };
///
/// assert_eq!(tree, expected);
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct TreeBuilder<T> {
    /// Data in the tree being built.
    tree: Tree<T>,
    /// References to parent nodes of the current node being constructed.
    parents: Vec<NonMaxUsize>,
    /// Reference to last sibling inserted.
    sibling: Option<NonMaxUsize>,
    /// The current cursor.
    cursor: usize,
}

impl<T> TreeBuilder<T> {
    /// Construct a new tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root");
    ///
    /// tree.open("child");
    /// tree.token("token", 5);
    /// tree.close()?;
    ///
    /// tree.open("child2");
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
    ///         "child2"
    ///     }
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok(()) }
    /// ```
    pub const fn new() -> Self {
        TreeBuilder {
            tree: Tree::new(),
            parents: Vec::new(),
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root");
    ///
    /// tree.open("child");
    /// tree.close()?;
    ///
    /// tree.open("child");
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub fn open(&mut self, data: T) -> Id {
        let id = self.insert(data, Kind::Node, Span::point(self.cursor));
        self.parents.push(id);
        Id(id)
    }

    /// End a node being built.
    ///
    /// This call must be balanced with a prior call to [TreeBuilder::open]. If
    /// not this will result in an [TreeError::CloseError] being raised.
    ///
    /// This will pop a value of the stack, and set that value as the next
    /// sibling which will be used with [TreeBuilder::open].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root");
    ///
    /// tree.open("child");
    /// tree.close()?;
    ///
    /// tree.open("child");
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub fn close(&mut self) -> Result<(), TreeError> {
        let head = match self.parents.pop() {
            Some(head) => head,
            None => return Err(TreeError::CloseError),
        };

        self.sibling = Some(head);

        if let Some(parent) = self.parents.last().copied() {
            if let Some(node) = self.tree.get_mut(head) {
                let end = node.span.end;

                if let Some(parent) = self.tree.get_mut(parent) {
                    parent.span.end = end;
                }
            }
        }

        Ok(())
    }

    /// Declare a token with the specified `value` and a corresponding `len`.
    ///
    /// A token is always a terminating element without children.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("child");
    /// tree.token("lit", 3);
    /// tree.close()?;
    ///
    /// # Ok(()) }
    /// ```
    pub fn token(&mut self, value: T, len: usize) -> Id {
        let start = self.cursor;

        if len > 0 {
            self.cursor = self.cursor.checked_add(len).expect("cursor out of bounds");
            self.tree.span.end = self.cursor;
        }

        let id = self.insert(value, Kind::Token, Span::new(start, self.cursor));
        self.sibling = Some(id);
        Id(id)
    }

    /// Get a checkpoint corresponding to the current position in the tree.
    ///
    /// # Panics
    ///
    /// This panics if the number of nodes are too many to fit in a vector on
    /// your architecture. This corresponds to [usize::MAX].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// let c = tree.checkpoint();
    /// tree.open("child");
    /// tree.token("lit", 3);
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
    /// # Ok(()) }
    /// ```
    pub fn checkpoint(&self) -> Id {
        Id(NonMaxUsize::new(self.tree.len()).expect("ran out of ids"))
    }

    /// Insert a node that wraps from the given checkpointed location.
    ///
    /// The checkpoint being closed *must* be a sibling. Otherwise a
    /// [TreeError::CloseAtError] will be raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// let c = tree.checkpoint();
    /// tree.token("lit", 3);
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
    /// # Ok(()) }
    /// ```
    ///
    /// More complex example:
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// let c = tree.checkpoint();
    ///
    /// tree.open("number");
    /// tree.token("lit", 3);
    /// tree.close()?;
    ///
    /// tree.token("whitespace", 1);
    ///
    /// tree.open("number");
    /// tree.token("lit", 2);
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
    /// # Ok(()) }
    /// ```
    ///
    /// Adding a token after a checkpoint:
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// let c = tree.checkpoint();
    /// tree.open("child");
    /// tree.token("lit", 3);
    /// tree.close()?;
    /// tree.close_at(c, "root")?;
    /// tree.token("sibling", 3);
    ///
    /// let tree = tree.build()?;
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
    /// # Ok(()) }
    /// ```
    pub fn close_at(&mut self, id: Id, data: T) -> Result<Id, TreeError> {
        let child = NonMaxUsize::new(self.tree.len()).expect("ran out of ids");

        let links = match self.tree.get_mut(id.0) {
            Some(links) => links,
            None => {
                let id = self.insert(data, Kind::Node, Span::point(self.cursor));
                self.sibling = Some(id);
                return Ok(Id(id));
            }
        };

        // We attempt to restructure the current node, unless it is an immediate
        // and sole sibling - which it is if it was just inserted into
        // next.sibling.
        //
        // The primary purpose of restructuring is to calculate the span of the
        // range of nodes we are wrapping.
        let needs_restructuring = self.sibling != Some(id.0);

        let removed = mem::replace(
            links,
            Links {
                data,
                kind: Kind::Node,
                next: None,
                first: Some(child),
                span: links.span,
            },
        );

        if needs_restructuring {
            let start = links.span.start;
            self.restructure_close_at(id.0, start, &removed)?;
        }

        self.tree.push(removed);

        // The current sibling is the newly replaced node in the tree.
        self.sibling = Some(id.0);
        Ok(Id(id.0))
    }

    // Adjust span to encapsulate all children and check that we just inserted
    // the checkpointed node in the right location which should be the tail
    // sibling of the replaced node.
    fn restructure_close_at(
        &mut self,
        id: NonMaxUsize,
        start: usize,
        removed: &Links<T>,
    ) -> Result<(), TreeError> {
        let sibling = self
            .tree
            .node_at(self.sibling)
            .ok_or(TreeError::CloseAtError)?;

        let node = self
            .tree
            .node_at(removed.next)
            .and_then(|n| n.siblings().last());

        if let Some(node) = node {
            let span = Span::new(start, node.span().end);

            if !sibling.is_same(&node) {
                return Err(TreeError::CloseAtError);
            }

            if let Some(node) = self.tree.get_mut(id) {
                node.span = span;
            }
        } else if !sibling.is_same_as_links(removed) {
            return Err(TreeError::CloseAtError);
        }

        Ok(())
    }

    /// Build a [Tree] from the current state of the builder.
    ///
    /// This requires the stack in the builder to be empty. Otherwise a
    /// [TreeError::BuildError] will be raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("child");
    /// tree.token("number", 3);
    /// tree.close()?;
    /// tree.open("child");
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let expected = syntree::tree! {
    ///     "child" => {
    ///         ("number", 3)
    ///     },
    ///     "child",
    /// };
    ///
    /// assert_eq!(tree, expected);
    /// # Ok(()) }
    /// ```
    ///
    /// If a tree is unbalanced during construction, building will fail with an error:
    ///
    /// ```
    /// use syntree::{TreeError, Span, TreeBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("number");
    /// tree.token("lit", 3);
    /// tree.close()?;
    ///
    /// tree.open("number");
    ///
    /// // "number" is left open.
    /// assert!(matches!(tree.build(), Err(TreeError::BuildError)));
    /// # Ok(()) }
    /// ```
    pub fn build(self) -> Result<Tree<T>, TreeError> {
        if !self.parents.is_empty() {
            return Err(TreeError::BuildError);
        }

        Ok(self.tree)
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, kind: Kind, span: Span) -> NonMaxUsize {
        let new = NonMaxUsize::new(self.tree.len()).expect("ran out of ids");

        let prev = std::mem::replace(&mut self.sibling, None);
        let parent = self.parents.last().copied();

        self.tree.push(Links {
            data,
            kind,
            next: None,
            first: None,
            span,
        });

        if let Some(node) = self.tree.links_at_mut(prev) {
            node.next = Some(new);
        }

        if let Some(node) = self.tree.links_at_mut(parent) {
            if node.first.is_none() {
                node.first = Some(new);
            }

            node.span.end = span.end;
        }

        new
    }
}

impl<T> Default for TreeBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
