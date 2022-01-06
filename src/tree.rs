use std::fmt;

use crate::links::Links;
use crate::non_max::NonMaxUsize;
use crate::{Children, Node, Walk, WalkEvents};

/// The kind of a node in the [Tree].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// A node.
    Node,
    /// The token and a corresponding span.
    Token,
}

/// A syntax tree.
#[derive(Clone)]
pub struct Tree<T> {
    pub(crate) tree: Vec<Links<T>>,
}

impl<T> Tree<T> {
    /// Construct a new empty tree.
    pub(crate) const fn new() -> Self {
        Self { tree: Vec::new() }
    }

    /// Check if the current tree is empty. In that it doesn't have any
    /// childrens at the root of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::<()>::new();
    /// let tree = tree.build()?;
    /// assert!(tree.is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Get all root nodes in the tree.
    ///
    /// See [Children] for documentation.
    pub fn children(&self) -> Children<'_, T> {
        Children::new(self.first())
    }

    /// Walk the tree forwards in a depth-first fashion visiting every node once.
    ///
    /// See [Walk] for documentation.
    pub fn walk(&self) -> Walk<'_, T> {
        Walk::new(self.first())
    }

    /// Walk the tree forwards in a depth-first fashion emitting events
    /// indicating how the tree is being traversed.
    ///
    /// See [WalkEvents] for documentation.
    pub fn walk_events(&self) -> WalkEvents<'_, T> {
        WalkEvents::new(self.first())
    }

    /// Get the first child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root",
    ///     "root2"
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root");
    /// # Ok(()) }
    /// ```
    pub fn first(&self) -> Option<Node<'_, T>> {
        self.node_at(NonMaxUsize::new(0))
    }

    /// The total number of elements in the tree.
    pub(crate) fn len(&self) -> usize {
        self.tree.len()
    }

    /// Get a mutable reference to an element in the tree.
    pub(crate) fn get_mut(&mut self, id: NonMaxUsize) -> Option<&mut Links<T>> {
        self.tree.get_mut(id.get())
    }

    /// Push a new element onto the tree.
    pub(crate) fn push(&mut self, links: Links<T>) {
        self.tree.push(links);
    }

    /// Optionally get the links at the given location.
    pub(crate) fn links_at_mut(&mut self, index: Option<NonMaxUsize>) -> Option<&mut Links<T>> {
        self.tree.get_mut(index?.get())
    }

    /// Construct a node at the given location.
    pub(crate) fn node_at(&self, index: Option<NonMaxUsize>) -> Option<Node<'_, T>> {
        let cur = self.tree.get(index?.get())?;
        Some(Node::new(cur, &self.tree))
    }
}

impl<T> PartialEq for Tree<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.walk().with_depths().eq(other.walk().with_depths())
    }
}

impl<T> Eq for Tree<T> where T: Eq {}

impl<T> fmt::Debug for Tree<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.debug_tuple("Tree").field(&List(self)).finish();

        struct List<'a, T>(&'a Tree<T>);

        impl<T> fmt::Debug for List<'_, T>
        where
            T: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.walk().with_depths()).finish()
            }
        }
    }
}
