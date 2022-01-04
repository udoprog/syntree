use crate::non_max::NonMaxUsize;
use crate::tree::{Links, Node};
use crate::{Kind, Span};

/// Iterator over the children of a node or tree. This includes [Kind::Token]
/// nodes.
///
/// See [Tree::children_with_tokens][crate::Tree::children_with_tokens].
pub struct ChildrenWithTokens<'a, T> {
    pub(crate) tree: &'a [Links<T>],
    pub(crate) start: Option<NonMaxUsize>,
    pub(crate) end: Option<NonMaxUsize>,
}

impl<'a, T> ChildrenWithTokens<'a, T> {
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
    /// let mut it = tree.children_with_tokens();
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
                Kind::Node => node.children_with_tokens().span(),
                Kind::Token(a) => Some(a),
            };

            if let Some(u) = u {
                output = Some(output.map(|s| s.join(u)).unwrap_or(u));
            }
        }

        output
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
    /// let mut it = tree.children_with_tokens();
    /// assert_eq!(it.next_node().map(|n| *n.data()), Some("child1"));
    /// assert_eq!(it.next().map(|n| *n.data()), Some("t2"));
    /// # Ok(()) }
    /// ```
    pub fn next_node(&mut self) -> Option<Node<'a, T>> {
        loop {
            let node = self.tree.get(self.start?.get())?;
            self.start = node.next;

            if !matches!(node.kind, Kind::Node) {
                continue;
            }

            return Some(Node {
                node,
                tree: self.tree,
            });
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
    /// let mut it = tree.children_with_tokens();
    /// assert_eq!(it.next_back_node().map(|n| *n.data()), Some("child3"));
    /// assert_eq!(it.next_back().map(|n| *n.data()), Some("t3"));
    /// # Ok(()) }
    /// ```
    pub fn next_back_node(&mut self) -> Option<Node<'a, T>> {
        loop {
            let node = self.tree.get(self.end?.get())?;
            self.end = node.prev;

            if !matches!(node.kind, Kind::Node) {
                continue;
            }

            return Some(Node {
                node,
                tree: self.tree,
            });
        }
    }
}

impl<'a, T> Iterator for ChildrenWithTokens<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.start?.get())?;
        self.start = node.next;

        Some(Node {
            node,
            tree: self.tree,
        })
    }
}

impl<'a, T> DoubleEndedIterator for ChildrenWithTokens<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.end?.get())?;
        self.end = node.prev;

        Some(Node {
            node,
            tree: self.tree,
        })
    }
}

impl<'a, T> Clone for ChildrenWithTokens<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for ChildrenWithTokens<'a, T> {}
