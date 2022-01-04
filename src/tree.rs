use std::fmt;

use crate::Span;

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
    node: &'a Links<T>,
    tree: &'a [Links<T>],
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

    /// Calculate the span of the node.
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
    /// tree.token("number", Span::new(0, 5));
    /// tree.end_node()?;
    ///
    /// tree.start_node("ident");
    /// tree.token("ident", Span::new(5, 7));
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.span(), Some(Span::new(0, 7)));
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> Option<Span> {
        self.children().span()
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
    /// tree.token("token", Span::new(0, 5));
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
            if self.node.first == usize::MAX {
                debug_assert_eq!(self.node.last, usize::MAX);
            }
        }

        self.node.first == usize::MAX
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

    fn node_at(&self, index: usize) -> Option<Node<'a, T>> {
        let cur = self.tree.get(index)?;

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
    pub(crate) prev: usize,
    pub(crate) next: usize,
    pub(crate) first: usize,
    pub(crate) last: usize,
}

/// A syntax tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree<T> {
    tree: Vec<Links<T>>,
    last: usize,
}

impl<T> Tree<T> {
    /// Construct a new tree.
    pub(crate) fn new(tree: Vec<Links<T>>, last: usize) -> Self {
        Self { tree, last }
    }

    /// Calculate the span of the node.
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
    /// tree.token("number", Span::new(0, 5));
    /// tree.end_node()?;
    ///
    /// tree.start_node("ident");
    /// tree.token("ident", Span::new(5, 7));
    /// tree.end_node()?;
    ///
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.span(), Some(Span::new(0, 7)));
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> Option<Span> {
        self.children().span()
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
        self.last == usize::MAX
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
            start: 0,
            end: self.last,
        }
    }

    /// Get the first child node in the tree.
    pub fn first(&self) -> Option<Node<'_, T>> {
        self.node_at(0)
    }

    /// Get the last child node in the tree.
    pub fn last(&self) -> Option<Node<'_, T>> {
        self.node_at(self.last)
    }

    fn node_at(&self, index: usize) -> Option<Node<'_, T>> {
        let cur = self.tree.get(index)?;

        Some(Node {
            node: cur,
            tree: &self.tree,
        })
    }
}

/// Access a sub tree.
pub struct Children<'a, T> {
    tree: &'a [Links<T>],
    start: usize,
    end: usize,
}

impl<'a, T> Children<'a, T> {
    /// Calculate the span of the remaining nodes in the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.start_node("number");
    /// tree.token("number", Span::new(0, 5));
    /// tree.end_node()?;
    ///
    /// tree.start_node("ident");
    /// tree.token("ident", Span::new(5, 7));
    /// tree.end_node()?;
    ///
    /// let tree = tree.build()?;
    /// let mut it = tree.children();
    ///
    /// it.next();
    ///
    /// assert_eq!(it.span(), Some(Span::new(5, 7)));
    /// # Ok(()) }
    /// ```
    pub fn span(self) -> Option<Span> {
        let mut output = None::<Span>;

        for node in self {
            let u = match node.kind() {
                Kind::Node => node.children().span(),
                Kind::Token(a) => Some(a),
            };

            if let Some(u) = u {
                output = Some(output.map(|s| s.join(u)).unwrap_or(u));
            }
        }

        output
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.start)?;
        self.start = node.next;

        Some(Node {
            node,
            tree: self.tree,
        })
    }
}

impl<'a, T> DoubleEndedIterator for Children<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.end)?;
        self.end = node.prev;

        Some(Node {
            node,
            tree: self.tree,
        })
    }
}

impl<'a, T> Clone for Children<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Children<'a, T> {}
