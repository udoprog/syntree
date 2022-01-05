use crate::links::Links;
use crate::non_max::NonMaxUsize;
use crate::Node;

/// An iterator that walks over the entire tree, visiting every node exactly
/// once.
///
/// See [Tree::walk][crate::Tree::walk] or [Node::walk].
pub struct Walk<'a, T> {
    tree: &'a [Links<T>],
    // The current node.
    start: Option<NonMaxUsize>,
    // Parent nodes.
    parents: Vec<NonMaxUsize>,
}

impl<'a, T> Walk<'a, T> {
    pub(crate) fn new(tree: &'a [Links<T>], start: Option<NonMaxUsize>) -> Self {
        Self {
            tree,
            start,
            parents: Vec::new(),
        }
    }

    fn step(&mut self, id: NonMaxUsize, links: &Links<T>) -> Option<NonMaxUsize> {
        if let Some(first) = links.first {
            self.parents.push(id);
            return Some(first);
        }

        if let Some(next) = links.next {
            return Some(next);
        }

        while let Some(parent) = self.parents.pop() {
            if let Some(id) = self.tree.get(parent.get())?.next {
                return Some(id);
            }
        }

        None
    }

    /// Convert this iterator into one which includes depths.
    ///
    /// # Examples
    ///
    /// Walking a whole tree:
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
    /// let nodes = tree.walk().with_depths().map(|(d, n)| (d, *n.data())).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec![(0, "root"), (1, "c1"), (2, "c2"), (2, "c3"), (2, "c4"), (1, "c5"), (1, "c6")]);
    /// # Ok(()) }
    /// ```
    ///
    /// Walking a sub-tree:
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
    /// let nodes = root.walk().with_depths().map(|(d, n)| (d, *n.data())).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec![(0, "c1"), (1, "c2"), (1, "c3"), (1, "c4"), (0, "c5"), (0, "c6")]);
    /// # Ok(()) }
    /// ```
    pub fn with_depths(self) -> WithDepths<'a, T> {
        WithDepths { iter: self }
    }

    /// Get the next node including the depth which it is located at.
    pub fn next_with_depth(&mut self) -> Option<(usize, Node<'a, T>)> {
        let depth = self.parents.len();
        Some((depth, self.next()?))
    }
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.start.take()?;
        let links = self.tree.get(id.get())?;

        if let Some(id) = self.step(id, links) {
            self.start = Some(id);
        }

        Some(Node::new(links, self.tree))
    }
}

/// An iterator that walks over the entire tree, visiting every node exactly
/// once.
///
/// See [Walk::with_depths].
pub struct WithDepths<'a, T> {
    iter: Walk<'a, T>,
}

impl<'a, T> Iterator for WithDepths<'a, T> {
    type Item = (usize, Node<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_with_depth()
    }
}
