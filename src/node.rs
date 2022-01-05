use std::fmt;

use crate::links::Links;
use crate::non_max::NonMaxUsize;
use crate::tree::Kind;
use crate::{Children, Span, Walk, WalkEvents};

/// A node in the tree.
pub struct Node<'a, T> {
    links: &'a Links<T>,
    tree: &'a [Links<T>],
}

impl<'a, T> Node<'a, T> {
    pub(crate) fn new(node: &'a Links<T>, tree: &'a [Links<T>]) -> Self {
        Self { links: node, tree }
    }

    /// Test if this node is the same as another node.
    ///
    /// This is a cheap pointer comparison.
    pub(crate) fn is_same(&self, other: &Node<'a, T>) -> bool {
        std::ptr::eq(self.links, other.links)
    }

    /// Test if this node is the same as another set of links.
    ///
    /// This is a cheap pointer comparison.
    pub(crate) fn is_same_as_links(&self, links: &Links<T>) -> bool {
        std::ptr::eq(self.links, links)
    }

    /// Access the data associated with the node.
    pub fn value(&self) -> &'a T {
        &self.links.data
    }

    /// Access the kind of the node.
    pub fn kind(&self) -> Kind {
        self.links.kind
    }

    /// Get the span of the current node. The span of a node is the complete
    /// span of all its children.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.span(), Span::new(0, 8));
    ///
    /// let root2 = root.next().ok_or("missing second root")?;
    /// assert_eq!(root2.span(), Span::new(8, 13));
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> Span {
        self.links.span
    }

    /// Check if the current node is empty. In that it doesn't have any
    /// children.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = syntree::tree! {
    ///     "root",
    ///     "root2" => {
    ///         ("token", 5)
    ///     }
    /// };
    ///
    /// let first = tree.first().ok_or("missing first root node")?;
    /// let last = first.next().ok_or("missing last root node")?;
    ///
    /// assert!(first.is_empty());
    /// assert!(!last.is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.links.first.is_none()
    }

    /// Get an iterator over the children of this node.
    ///
    /// See [Children] for documentation.
    pub fn children(&self) -> Children<'a, T> {
        Children::new(self.tree, self.links.first)
    }

    /// Walk the subtree forward starting with the first child of the current
    /// node.
    ///
    /// See [Walk] for documentation.
    pub fn walk(&self) -> Walk<'a, T> {
        Walk::new(self.tree, self.links.first)
    }

    /// Walk the node forwards in a depth-first fashion emitting events
    /// indicating how the rest of the tree is being traversed.
    ///
    /// See [WalkEvents] for documentation.
    pub fn walk_events(&self) -> WalkEvents<'_, T> {
        WalkEvents::new(self.tree.as_ref(), self.links.first)
    }

    /// Get the first child node.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(*number.value(), "number");
    /// # Ok(()) }
    /// ```
    pub fn first(&self) -> Option<Node<'a, T>> {
        self.node_at(self.links.first)
    }

    /// Get the next sibling.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing second root")?;
    /// assert_eq!(*number.value(), "number");
    ///
    /// let ident = number.next().ok_or("missing second root")?;
    /// assert_eq!(*ident.value(), "ident");
    /// # Ok(()) }
    /// ```
    pub fn next(&self) -> Option<Node<'a, T>> {
        self.node_at(self.links.next)
    }

    fn node_at(&self, index: Option<NonMaxUsize>) -> Option<Node<'a, T>> {
        let cur = self.tree.get(index?.get())?;

        Some(Self {
            links: cur,
            tree: self.tree,
        })
    }
}

impl<'a, T> fmt::Debug for Node<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.links.data)
            .field("kind", &self.links.kind)
            .finish()
    }
}

impl<'a, T> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Node<'a, T> {}

impl<'a, T> PartialEq for Node<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.links.data == other.links.data && self.links.kind == other.links.kind
    }
}

impl<'a, T> Eq for Node<'a, T> where T: Eq {}
