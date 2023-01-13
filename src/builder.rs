use std::cell::Cell;
use std::mem::replace;
use std::rc::Rc;

use crate::links::Links;
use crate::non_max::NonMax;
use crate::span::{usize_to_index, Index};
use crate::{Kind, Span, Tree, TreeError};

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::open] or [TreeBuilder::token].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(pub(crate) NonMax);

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::checkpoint].
///
/// This can be used as a checkpoint in [TreeBuilder::close_at], and a
/// checkpoint can be fetched up front from [TreeBuilder::checkpoint].
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Checkpoint(Rc<Cell<CheckpointData>>);

impl Id {
    pub(crate) const fn new(id: NonMax) -> Self {
        Self(id)
    }
}

/// The parent of the checkpoint.
#[derive(Debug, Clone, Copy)]
struct CheckpointData {
    // The node being wrapped by the checkpoint.
    node: NonMax,
    // The parent node of the context being checkpointed.
    parent: Option<NonMax>,
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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut tree = TreeBuilder::new();
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
    tree: Tree<T, S>,
    /// References to parent nodes of the current node being constructed.
    parents: Vec<NonMax>,
    /// The last checkpoint that was handed out.
    checkpoint: Option<Checkpoint>,
    /// Reference to last sibling inserted.
    sibling: Option<NonMax>,
    /// The current cursor.
    cursor: Index,
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
            checkpoint: None,
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
    /// tree.open("root")?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub fn open(&mut self, data: T) -> Result<Id, TreeError> {
        let id = self.insert(data, Kind::Node, Span::point(self.cursor))?;
        self.parents.push(id);
        Ok(Id(id))
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
    /// tree.open("root")?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.open("child")?;
    /// tree.close()?;
    ///
    /// tree.close()?;
    /// # Ok(()) }
    /// ```
    pub fn close(&mut self) -> Result<(), TreeError> {
        let head = self.parents.pop().ok_or(TreeError::CloseError)?;
        self.sibling = Some(head);

        if let Some(&parent) = self.parents.last() {
            let node = self
                .tree
                .get_mut(head)
                .ok_or(TreeError::MissingNode(Id(head)))?;
            let end = node.span.end;
            let parent = self
                .tree
                .get_mut(parent)
                .ok_or(TreeError::MissingNode(Id(parent)))?;
            parent.span.end = end;
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
    /// tree.open("child")?;
    /// tree.token("lit", 4)?;
    /// tree.close()?;
    ///
    /// # Ok(()) }
    /// ```
    pub fn token(&mut self, value: T, len: usize) -> Result<Id, TreeError> {
        let start = self.cursor;

        if len > 0 {
            self.cursor = usize_to_index(len)
                .and_then(|len| self.cursor.checked_add(len))
                .ok_or(TreeError::Overflow)?;
            self.tree.span_mut().end = self.cursor;
        }

        let id = self.insert(value, Kind::Token, Span::new(start, self.cursor))?;
        self.sibling = Some(id);

        if len > 0 {
            self.tree.push_index(self.cursor, id);
        }

        Ok(Id(id))
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
    /// # Ok(()) }
    /// ```
    pub fn checkpoint(&mut self) -> Result<Checkpoint, TreeError> {
        let node = NonMax::new(self.tree.len()).ok_or(TreeError::Overflow)?;

        if let Some(c) = &self.checkpoint {
            if c.0.get().node == node {
                return Ok(c.clone());
            }
        }

        let c = Checkpoint(Rc::new(Cell::new(CheckpointData {
            node,
            parent: self.parents.last().copied(),
        })));

        self.checkpoint = Some(c.clone());
        Ok(c)
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
    /// # Ok(()) }
    /// ```
    pub fn close_at(&mut self, c: &Checkpoint, data: T) -> Result<Id, TreeError> {
        let c = &*c.0;
        let id = c.get().node;

        if c.get().parent.as_ref() != self.parents.last() {
            return Err(TreeError::CloseAtError);
        }

        let next_id = NonMax::new(self.tree.len()).ok_or(TreeError::Overflow)?;

        let Some(links) = self.tree.get_mut(id) else {
            let new_id = self.insert(data, Kind::Node, Span::point(self.cursor))?;
            self.sibling = Some(new_id);
            debug_assert_eq!(new_id, id, "new id should match the expected id");
            return Ok(Id(new_id));
        };

        let parent = replace(&mut links.parent, Some(next_id));
        let prev = replace(&mut links.prev, None);

        // Restructuring is necessary to calculate the full span of the newly
        // inserted node and update parent references to point to the newly
        // inserted node.
        let (last, span) = if let Some(next) = links.next {
            let span = links.span;
            let (last, end) = restructure_close_at(&mut self.tree, next_id, next)?;
            (Some(last), Span::new(span.start, end))
        } else {
            (Some(id), links.span)
        };

        let added = Links {
            data,
            kind: Kind::Node,
            span,
            prev,
            parent,
            next: None,
            first: Some(id),
            last,
        };

        if let Some(parent) = parent.and_then(|id| self.tree.get_mut(id)) {
            if parent.first == Some(id) {
                parent.first = Some(next_id);
            }

            if parent.last == Some(id) {
                parent.last = Some(next_id);
            }
        }

        if let Some(prev) = prev.and_then(|id| self.tree.get_mut(id)) {
            prev.next = Some(next_id);
        }

        // If we're replacing the first node of the tree, the newly inserted
        // node should be set as the first node.
        if self.tree.first_id() == Some(id) {
            let (first, _) = self.tree.links_mut();
            *first = Some(next_id);
        }

        // Do necessary accounting.
        self.tree.push(added);
        self.sibling = Some(next_id);

        c.set(CheckpointData {
            node: next_id,
            parent,
        });

        Ok(Id(next_id))
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
    /// tree.open("number")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    ///
    /// tree.open("number")?;
    ///
    /// // "number" is left open.
    /// assert!(matches!(tree.build(), Err(TreeError::BuildError)));
    /// # Ok(()) }
    /// ```
    pub fn build(self) -> Result<Tree<T, S>, TreeError> {
        if !self.parents.is_empty() {
            return Err(TreeError::BuildError);
        }

        Ok(self.tree)
    }

    /// Insert a new node.
    fn insert(&mut self, data: T, kind: Kind, span: Span) -> Result<NonMax, TreeError> {
        let new = NonMax::new(self.tree.len()).ok_or(TreeError::Overflow)?;

        let prev = self.sibling.take();
        let parent = self.parents.last().copied();

        self.tree.push(Links {
            data,
            kind,
            span,
            parent,
            prev,
            next: None,
            first: None,
            last: None,
        });

        if let Some(id) = parent {
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

impl<T> Default for TreeBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Adjust span to encapsulate all children and check that we just inserted the
// checkpointed node in the right location which should be the tail sibling of
// the replaced node.
fn restructure_close_at<T>(
    tree: &mut Tree<T, S>,
    parent_id: NonMax,
    next: NonMax,
) -> Result<(NonMax, Index), TreeError> {
    let mut links = tree.get_mut(next).ok_or(TreeError::MissingNode(Id(next)))?;
    let mut last = (next, links.span.end);
    links.parent = Some(parent_id);

    while let Some(next) = links.next {
        links = tree.get_mut(next).ok_or(TreeError::MissingNode(Id(next)))?;
        last = (next, links.span.end);
        links.parent = Some(parent_id);
    }

    Ok(last)
}
