use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node, Walk};
use crate::{Kind, Span, WithoutTokens};

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
    /// tree.open("number");
    /// tree.token("number", 5);
    /// tree.close()?;
    ///
    /// tree.open("ident");
    /// tree.token("ident", 2);
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    /// let mut it = tree.children();
    ///
    /// it.next();
    ///
    /// assert_eq!(it.span(), Some(Span::new(5, 7)));
    /// # Ok(()) }
    /// ```
    pub fn span(mut self) -> Option<Span> {
        let first = self.next().map(|n| n.span());
        let last = self.next_back().map(|n| n.span());

        match (first, last) {
            (Some(first), Some(last)) => Some(first.join(last)),
            (first, last) => first.or(last),
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
    /// assert_eq!(nodes, vec!["c6", "c5", "c4", "c3", "c2", "c1", "root"]);
    /// # Ok(()) }
    /// ```
    pub fn walk(self) -> Walk<'a, T> {
        Walk {
            tree: self.tree,
            range: self.range(self.start, self.end),
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

    fn range(
        &self,
        start: Option<NonMaxUsize>,
        mut end: Option<NonMaxUsize>,
    ) -> Option<(usize, usize)> {
        let start = start?;

        while let Some(last) = self.tree.get(end?.get())?.last {
            end = Some(last);
        }

        Some((start.get(), end?.get()))
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
