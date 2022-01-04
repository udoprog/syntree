use std::fmt;

use crate::non_max::NonMaxUsize;
use crate::Span;

mod children;
pub use self::children::Children;

mod children_with_tokens;
pub use self::children_with_tokens::ChildrenWithTokens;

/// The kind of a node in the [Tree].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// A node.
    Node,
    /// The token and a corresponding span.
    Token(Span),
}

/// A node in the tree.
pub struct Node<'a, T> {
    pub(crate) node: &'a Links<T>,
    pub(crate) tree: &'a [Links<T>],
}

impl<'a, T> Node<'a, T> {
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
        if let Some(span) = self.children_with_tokens().span() {
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
    pub fn children_with_tokens(&self) -> ChildrenWithTokens<'a, T> {
        ChildrenWithTokens {
            tree: self.tree,
            start: self.node.first,
            end: self.node.last,
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Links<T> {
    pub(crate) data: T,
    pub(crate) kind: Kind,
    pub(crate) prev: Option<NonMaxUsize>,
    pub(crate) next: Option<NonMaxUsize>,
    pub(crate) first: Option<NonMaxUsize>,
    pub(crate) last: Option<NonMaxUsize>,
}

/// A syntax tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree<T> {
    tree: Vec<Links<T>>,
    last: Option<NonMaxUsize>,
}

impl<T> Tree<T> {
    /// Construct a new tree.
    pub(crate) fn new(tree: Vec<Links<T>>, last: Option<NonMaxUsize>) -> Self {
        Self { tree, last }
    }

    /// Calculate the span of the tree. If there is no span information
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
    /// assert_eq!(tree.span(), Span::new(0, 7));
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> Span {
        if let Some(span) = self.children_with_tokens().span() {
            span
        } else {
            Span::new(0, usize::MAX)
        }
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
        self.last.is_none()
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
    /// tree.start_node("root1");
    /// tree.start_node("child1");
    /// tree.end_node()?;
    /// tree.end_node()?;
    ///
    /// tree.start_node("root2");
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let mut it = tree.children();
    ///
    /// assert_eq!(it.next().map(|n| *n.data()), Some("root1"));
    /// assert_eq!(it.next().map(|n| *n.data()), Some("root2"));
    /// assert!(it.next().is_none());
    /// # Ok(()) }
    /// ```
    pub fn children_with_tokens(&self) -> ChildrenWithTokens<'_, T> {
        ChildrenWithTokens {
            tree: self.tree.as_slice(),
            start: NonMaxUsize::new(0),
            end: self.last,
        }
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
    /// tree.start_node("root1");
    /// tree.start_node("child1");
    /// tree.end_node()?;
    /// tree.end_node()?;
    ///
    /// tree.start_node("root2");
    /// tree.end_node()?;
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
            tree: self.tree.as_slice(),
            start: NonMaxUsize::new(0),
            end: self.last,
        }
    }

    /// Get the first child node in the tree.
    pub fn first(&self) -> Option<Node<'_, T>> {
        self.node_at(NonMaxUsize::new(0))
    }

    /// Get the last child node in the tree.
    pub fn last(&self) -> Option<Node<'_, T>> {
        self.node_at(self.last)
    }

    fn node_at(&self, index: Option<NonMaxUsize>) -> Option<Node<'_, T>> {
        let cur = self.tree.get(index?.get())?;

        Some(Node {
            node: cur,
            tree: &self.tree,
        })
    }
}
