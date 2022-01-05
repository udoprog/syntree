use crate::non_max::NonMaxUsize;
use crate::{Node, Span};

mod children;
pub use self::children::Children;

mod walk;
pub use self::walk::{Walk, WithDepths};

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

    /// The total number of elements in the tree.
    pub(crate) fn len(&self) -> usize {
        self.tree.len()
    }

    /// Get a reference to an element in the tree.
    pub(crate) fn get(&mut self, id: NonMaxUsize) -> Option<&Links<T>> {
        self.tree.get(id.get())
    }

    /// Get a mutable reference to an element in the tree.
    pub(crate) fn get_mut(&mut self, id: NonMaxUsize) -> Option<&mut Links<T>> {
        self.tree.get_mut(id.get())
    }

    /// Push a new element onto the tree.
    pub(crate) fn push(&mut self, links: Links<T>) {
        self.tree.push(links);
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
        self.walk().with_depths().eq(other.walk().with_depths())
    }
}

impl<T> Eq for Tree<T> where T: Eq {}
