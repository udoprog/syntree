use crate::non_max::NonMaxUsize;
use crate::{Node, Span};

mod children;
pub use self::children::Children;

mod walk;
pub use self::walk::{Walk, WalkWithDepths};

/// The kind of a node in the [Tree].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// A node.
    Node,
    /// The token and a corresponding span.
    Token,
}

#[derive(Debug, Clone)]
pub(crate) struct Links<T> {
    pub(crate) data: T,
    pub(crate) kind: Kind,
    pub(crate) span: Span,
    pub(crate) next: Option<NonMaxUsize>,
    pub(crate) first: Option<NonMaxUsize>,
}

/// A syntax tree.
#[derive(Debug, Clone)]
pub struct Tree<T> {
    tree: Box<[Links<T>]>,
}

impl<T> Tree<T> {
    /// Construct a new tree.
    pub(crate) fn new(tree: Box<[Links<T>]>) -> Self {
        Self { tree }
    }

    /// Check if the current tree is empty. In that it doesn't have any
    /// childrens at the root of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> anyhow::Result<()> {
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
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root1");
    /// tree.open("child1");
    /// tree.close()?;
    /// tree.close()?;
    ///
    /// tree.open("root2");
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    /// let mut it = tree.children();
    ///
    /// assert_eq!(it.next().map(|n| *n.data()), Some("root1"));
    /// assert_eq!(it.next().map(|n| *n.data()), Some("root2"));
    /// assert!(it.next().is_none());
    /// # Ok(()) }
    /// ```
    pub fn children(&self) -> Children<'_, T> {
        Children {
            tree: self.tree.as_ref(),
            start: NonMaxUsize::new(0),
        }
    }

    /// Walk the tree forwards in a depth-first fashion visiting every node once.
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
    /// let nodes = tree.walk().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["root", "c1", "c2", "c3", "c4", "c5", "c6"]);
    /// # Ok(()) }
    /// ```
    pub fn walk(&self) -> Walk<'_, T> {
        Walk::new(self.tree.as_ref(), NonMaxUsize::new(0), None)
    }

    /// Walk the tree forwards in a depth-first fashion visiting every node
    /// once including the depth of the node being walked.
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
    /// let nodes = tree.walk_with_depths().map(|(d, n)| (d, *n.data())).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec![(0, "root"), (1, "c1"), (2, "c2"), (2, "c3"), (2, "c4"), (1, "c5"), (1, "c6")]);
    /// # Ok(()) }
    /// ```
    pub fn walk_with_depths(&self) -> WalkWithDepths<'_, T> {
        WalkWithDepths::new(self.tree.as_ref(), NonMaxUsize::new(0), None)
    }

    /// Get the first child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     "first" => {
    ///         "child"
    ///     },
    ///     "last" => {
    ///         "child2"
    ///     }
    /// };
    ///
    /// assert_eq!(tree.first().map(|n| *n.data()), Some("first"));
    /// # Ok(()) }
    /// ```
    pub fn first(&self) -> Option<Node<'_, T>> {
        self.node_at(NonMaxUsize::new(0))
    }

    fn node_at(&self, index: Option<NonMaxUsize>) -> Option<Node<'_, T>> {
        let cur = self.tree.get(index?.get())?;
        Some(Node::new(cur, &self.tree))
    }
}

impl<T> PartialEq for Tree<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.walk_with_depths().eq(other.walk_with_depths())
    }
}

impl<T> Eq for Tree<T> where T: Eq {}
