use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node, Walk, WithoutTokens};
use crate::{Kind, Span};

/// Iterator over the children of a node or tree.
///
/// See [Tree::children][crate::Tree::children].
pub struct Children<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    pub(crate) start: Option<NonMaxUsize>,
    pub(crate) end: Option<NonMaxUsize>,
}

impl<'a, T> Children<'a, T> {
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
    /// let mut it = root.children().without_tokens();
    ///
    /// assert_eq!(it.next().map(|n| *n.data()), Some("child1"));
    /// assert_eq!(it.next().map(|n| *n.data()), Some("child2"));
    /// assert!(it.next().is_none());
    /// # Ok(()) }
    /// ```
    pub fn without_tokens(self) -> WithoutTokens<Self> {
        WithoutTokens::new(self)
    }

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
    /// tree.token("number", 5);
    /// tree.end_node()?;
    ///
    /// tree.start_node("ident");
    /// tree.token("ident", 2);
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
        let mut it = self.walk();

        let start = loop {
            if let Kind::Token(span) = it.next()?.kind() {
                break span;
            }
        };

        while let Some(node) = it.next_back() {
            if let Kind::Token(end) = node.kind() {
                return Some(Span::new(start.start, end.end));
            }
        }

        Some(start)
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
    /// let nodes = tree.children().walk().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["root", "c1", "c2", "c3", "c4", "c5", "c6"]);
    /// # Ok(()) }
    /// ```
    ///
    /// Walk backwards.
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
    /// let nodes = tree.children().walk().rev().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["root", "c6", "c5", "c1", "c4", "c3", "c2"]);
    /// # Ok(()) }
    /// ```
    pub fn walk(self) -> Walk<'a, T> {
        Walk {
            tree: self.tree,
            start: self.start,
            end: self.end,
        }
    }

    /// Get the next node from the iterator. This advances past all non-node
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     ("t1", 1),
    ///     "child1",
    ///     ("t2", 1),
    ///     "child2",
    ///     ("t3", 1),
    ///     "child3",
    ///     ("t4", 1)
    /// };
    ///
    /// let mut it = tree.children();
    /// assert_eq!(it.next_node().map(|n| *n.data()), Some("child1"));
    /// assert_eq!(it.next().map(|n| *n.data()), Some("t2"));
    /// # Ok(()) }
    /// ```
    pub fn next_node(&mut self) -> Option<Node<'a, T>> {
        loop {
            let node = self.tree.get(self.start?.get())?;
            self.start = node.next;

            if matches!(node.kind, Kind::Node) {
                return Some(Node::new(node, self.tree));
            }
        }
    }

    /// Get the next node from the iterator from the back. This advances past
    /// all non-node data.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     ("t1", 1),
    ///     "child1",
    ///     ("t2", 1),
    ///     "child2",
    ///     ("t3", 1),
    ///     "child3",
    ///     ("t4", 1)
    /// };
    ///
    /// let mut it = tree.children();
    /// assert_eq!(it.next_back_node().map(|n| *n.data()), Some("child3"));
    /// assert_eq!(it.next_back().map(|n| *n.data()), Some("t3"));
    /// # Ok(()) }
    /// ```
    pub fn next_back_node(&mut self) -> Option<Node<'a, T>> {
        loop {
            let node = self.tree.get(self.end?.get())?;
            self.end = node.prev;

            if matches!(node.kind, Kind::Node) {
                return Some(Node::new(node, self.tree));
            }
        }
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.start?.get())?;
        self.start = node.next;
        Some(Node::new(node, self.tree))
    }
}

impl<'a, T> DoubleEndedIterator for Children<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.end?.get())?;
        self.end = node.prev;
        Some(Node::new(node, self.tree))
    }
}

impl<'a, T> Clone for Children<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Children<'a, T> {}
