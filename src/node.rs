use std::fmt;

use crate::non_max::NonMaxUsize;
use crate::{
    tree::{Kind, Links},
    Children, Span, Walk, WalkRev,
};

/// A node in the tree.
pub struct Node<'a, T> {
    node: &'a Links<T>,
    tree: &'a [Links<T>],
}

impl<'a, T> Node<'a, T> {
    pub(crate) fn new(node: &'a Links<T>, tree: &'a [Links<T>]) -> Self {
        Self { node, tree }
    }

    /// Access the data associated with the node.
    pub fn data(&self) -> &'a T {
        &self.node.data
    }

    /// Access the kind of the node.
    pub fn kind(&self) -> Kind {
        self.node.kind
    }

    /// Calculate the span of the node. If there is no span information
    /// available, the range returned will be from 0 to [usize::MAX].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("root");
    ///
    /// tree.start_node("number");
    /// tree.token("number", 5);
    /// tree.end_node()?;
    ///
    /// tree.start_node("ident");
    /// tree.token("ident", 2);
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let root = tree.first().unwrap();
    ///
    /// assert_eq!(root.span(), Span::new(0, 7));
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> Span {
        if let Some(span) = self.children().span() {
            span
        } else {
            Span::new(0, usize::MAX)
        }
    }

    /// Check if the current node is empty. In that it doesn't have any
    /// children.
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("first");
    /// tree.end_node()?;
    ///
    /// tree.start_node("last");
    /// tree.token("token", 5);
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let first = tree.first().expect("expected first root node");
    /// let last = tree.last().expect("expected last root node");
    ///
    /// assert!(first.is_empty());
    /// assert!(!last.is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
        if cfg!(debug_assertions) {
            if self.node.first.is_none() {
                debug_assert_eq!(self.node.last, None);
            }
        }

        self.node.first.is_none()
    }

    /// Access the children to this node.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("root1");
    /// tree.start_node("child1");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child2");
    /// tree.end_node()?;
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let root = tree.first().expect("expected root node");
    ///
    /// let mut it = root.children();
    ///
    /// assert_eq!(it.next().map(|n| *n.data()), Some("child1"));
    /// assert_eq!(it.next().map(|n| *n.data()), Some("child2"));
    /// assert!(it.next().is_none());
    /// # Ok(()) }
    /// ```
    pub fn children(&self) -> Children<'a, T> {
        Children {
            tree: self.tree,
            start: self.node.first,
            end: self.node.last,
        }
    }

    /// Walk the rest of the tree forwards in a depth-first fashion.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "c1" => {
    ///             "c2",
    ///             "c3",
    ///             "c4",
    ///         },
    ///         "c5",
    ///         "c6"
    ///     }
    /// };
    ///
    /// let root = tree.first().expect("expected root node");
    ///
    /// let nodes = root.walk().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["c1", "c2", "c3", "c4", "c5", "c6"]);
    /// # Ok(()) }
    /// ```
    pub fn walk(&self) -> Walk<'a, T> {
        self.children().walk()
    }

    /// Walk the rest of the tree backwards in a depth-first fashion.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "c1" => {
    ///             "c2",
    ///             "c3",
    ///             "c4",
    ///         },
    ///         "c5",
    ///         "c6"
    ///     }
    /// };
    ///
    /// let root = tree.first().expect("expected root node");
    ///
    /// let nodes = root.walk_rev().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["c6", "c5", "c1", "c4", "c3", "c2"]);
    /// # Ok(()) }
    /// ```
    pub fn walk_rev(&self) -> WalkRev<'a, T> {
        self.children().walk_rev()
    }

    /// Get the first child node.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("root1");
    ///
    /// tree.start_node("child1");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child2");
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let root = tree.first().expect("expected root node");
    ///
    /// assert_eq!(root.first().map(|n| *n.data()), Some("child1"));
    /// # Ok(()) }
    /// ```
    pub fn first(&self) -> Option<Node<'a, T>> {
        self.node_at(self.node.first)
    }

    /// Get the last child node.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("root1");
    ///
    /// tree.start_node("child1");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child2");
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let root = tree.first().expect("expected root node");
    ///
    /// assert_eq!(root.last().map(|n| *n.data()), Some("child2"));
    /// # Ok(()) }
    /// ```
    pub fn last(&self) -> Option<Node<'a, T>> {
        self.node_at(self.node.last)
    }

    /// Get the next sibling.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("root1");
    ///
    /// tree.start_node("child1");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child2");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child3");
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let root = tree.first().expect("expected root node");
    ///
    /// let child = root.first().expect("expected first node");
    /// assert_eq!(*child.data(), "child1");
    /// assert_eq!(child.next().map(|n| *n.data()), Some("child2"));
    /// # Ok(()) }
    /// ```
    pub fn next(&self) -> Option<Node<'a, T>> {
        self.node_at(self.node.next)
    }

    /// Get the previous sibling.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("root1");
    ///
    /// tree.start_node("child1");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child2");
    /// tree.end_node()?;
    ///
    /// tree.start_node("child3");
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let root = tree.first().expect("expected root node");
    ///
    /// let child = root.last().expect("expected last node");
    /// assert_eq!(*child.data(), "child3");
    /// assert_eq!(child.prev().map(|n| *n.data()), Some("child2"));
    /// # Ok(()) }
    /// ```
    pub fn prev(&self) -> Option<Node<'a, T>> {
        self.node_at(self.node.prev)
    }

    fn node_at(&self, index: Option<NonMaxUsize>) -> Option<Node<'a, T>> {
        let cur = self.tree.get(index?.get())?;

        Some(Self {
            node: cur,
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
            .field("data", &self.node.data)
            .field("kind", &self.node.kind)
            .finish()
    }
}

impl<'a, T> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Node<'a, T> {}
