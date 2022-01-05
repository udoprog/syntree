use std::fmt;

use crate::non_max::NonMaxUsize;
use crate::tree::{Kind, Links};
use crate::{Children, Span, Walk, WalkWithDepths};

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
    /// tree.open("root");
    ///
    /// tree.open("number");
    /// tree.token("number", 5);
    /// tree.close()?;
    ///
    /// tree.open("ident");
    /// tree.token("ident", 2);
    /// tree.close()?;
    ///
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let root = tree.first().unwrap();
    ///
    /// assert_eq!(root.span(), Span::new(0, 7));
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> Span {
        self.node.span
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
    /// tree.open("first");
    /// tree.close()?;
    ///
    /// tree.open("last");
    /// tree.token("token", 5);
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let first = tree.first().expect("expected first root node");
    /// let last = first.next().expect("expected last root node");
    ///
    /// assert!(first.is_empty());
    /// assert!(!last.is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
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
    /// tree.open("root1");
    /// tree.open("child1");
    /// tree.close()?;
    ///
    /// tree.open("child2");
    /// tree.close()?;
    /// tree.close()?;
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
        }
    }

    /// Walk the subtree forward starting with the first child of the current
    /// node.
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
        Walk::new(self.tree, self.node.first, self.node.next)
    }

    /// Walk the subtree forward starting with the first child of the current
    /// node returning the depths of the nodes being walked relative to the
    /// current node.
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
    /// let nodes = root.walk_with_depths().map(|(d, n)| (d, *n.data())).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec![(0, "c1"), (1, "c2"), (1, "c3"), (1, "c4"), (0, "c5"), (0, "c6")]);
    /// # Ok(()) }
    /// ```
    pub fn walk_with_depths(&self) -> WalkWithDepths<'a, T> {
        WalkWithDepths::new(self.tree, self.node.first, self.node.next)
    }

    /// Get the first child node.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root1");
    ///
    /// tree.open("child1");
    /// tree.close()?;
    ///
    /// tree.open("child2");
    /// tree.close()?;
    ///
    /// tree.close()?;
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

    /// Get the next sibling.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root1");
    ///
    /// tree.open("child1");
    /// tree.close()?;
    ///
    /// tree.open("child2");
    /// tree.close()?;
    ///
    /// tree.open("child3");
    /// tree.close()?;
    ///
    /// tree.close()?;
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

    /// Get the parent node.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = syntree::tree! {
    ///     "root" => {
    ///         "child1"
    ///     }
    /// };
    ///
    /// let child1 = tree.first().and_then(|n| n.first()).expect("expected child node");
    /// assert_eq!(child1.parent().map(|n| *n.data()), Some("root"));
    /// # Ok(()) }
    /// ```
    pub fn parent(&self) -> Option<Node<'a, T>> {
        self.node_at(self.node.parent)
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
