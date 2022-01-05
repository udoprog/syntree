use crate::links::Links;
use crate::non_max::NonMaxUsize;
use crate::{Kind, Node, WithoutTokens};

/// Iterator over the children of a node or tree.
///
/// See [Tree::children][crate::Tree::children] or [Node::children].
pub struct Children<'a, T> {
    tree: &'a [Links<T>],
    node: Option<NonMaxUsize>,
}

impl<'a, T> Children<'a, T> {
    /// Construct a new child iterator.
    pub(crate) fn new(tree: &'a [Links<T>], node: Option<NonMaxUsize>) -> Self {
        Self { tree, node }
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
            let node = self.tree.get(self.node?.get())?;
            self.node = node.next;

            if matches!(node.kind, Kind::Node) {
                return Some(Node::new(node, self.tree));
            }
        }
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.tree.get(self.node?.get())?;
        self.node = node.next;
        Some(Node::new(node, self.tree))
    }
}

impl<'a, T> Clone for Children<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Children<'a, T> {}
