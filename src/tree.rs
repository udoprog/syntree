use crate::non_max::NonMaxUsize;
use crate::{Node, Span};

mod without_tokens;
pub use self::without_tokens::WithoutTokens;

mod children;
pub use self::children::Children;

mod walk;
pub use self::walk::Walk;

/// The kind of a node in the [Tree].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// A node.
    Node,
    /// The token and a corresponding span.
    Token(Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Links<T> {
    pub(crate) data: T,
    pub(crate) kind: Kind,
    pub(crate) parent: Option<NonMaxUsize>,
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
        if let Some(span) = self.children().span() {
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
    pub fn children(&self) -> Children<'_, T> {
        Children {
            tree: self.tree.as_slice(),
            start: NonMaxUsize::new(0),
            end: self.last,
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
    /// let nodes = tree.walk().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["root", "c1", "c2", "c3", "c4", "c5", "c6"]);
    /// # Ok(()) }
    /// ```
    ///
    /// Walking backwards.
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
    /// let nodes = tree.walk().rev().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["root", "c6", "c5", "c1", "c4", "c3", "c2"]);
    /// # Ok(()) }
    /// ```
    pub fn walk(&self) -> Walk<'_, T> {
        Walk {
            tree: self.tree.as_slice(),
            start: NonMaxUsize::new(0),
            end: self.last,
        }
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

    /// Get the last child node in the tree.
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
    /// assert_eq!(tree.last().map(|n| *n.data()), Some("last"));
    /// # Ok(()) }
    /// ```
    pub fn last(&self) -> Option<Node<'_, T>> {
        self.node_at(self.last)
    }

    fn node_at(&self, index: Option<NonMaxUsize>) -> Option<Node<'_, T>> {
        let cur = self.tree.get(index?.get())?;
        Some(Node::new(cur, &self.tree))
    }
}
