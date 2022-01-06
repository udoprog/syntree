use crate::{Event, Node, WalkEvents, WithoutTokens};

/// An iterator that walks over the entire tree, visiting every node exactly
/// once.
///
/// See [Tree::walk][crate::Tree::walk] or [Node::walk].
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// // Walk the entire tree.
/// assert_eq!(
///     tree.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "c1", "c2", "c3", "c4", "c5", "c6"]
/// );
///
/// // Walk from the root.
/// let root = tree.first().ok_or("missing root node")?;
/// assert_eq!(
///     root.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c1", "c2", "c3", "c4", "c5", "c6"]
/// );
///
/// // Walk from second child of the root. Note that the node itself is correctly excluded.
/// let c5 = root.first().and_then(|n| n.next()).ok_or("missing second child")?;
/// assert_eq!(c5.walk().map(|n| *n.value()).collect::<Vec<_>>(), Vec::<&str>::new());
/// # Ok(()) }
/// ```
pub struct Walk<'a, T> {
    iter: WalkEvents<'a, T>,
}

impl<'a, T> Walk<'a, T> {
    /// Construct a new walk.
    pub(crate) const fn new(node: Option<Node<'a, T>>) -> Self {
        Self {
            iter: WalkEvents::new(node),
        }
    }

    /// Convert this iterator into one which includes depths.
    ///
    /// See [WithDepths] for documentation.
    pub fn with_depths(self) -> WithDepths<'a, T> {
        WithDepths { iter: self }
    }

    /// Construct a [WithoutTokens] iterator from the remainder of this
    /// iterator. This filters out [Kind::Token][crate::Kind::Token] elements.
    ///
    /// See [WithoutTokens] for documentation.
    pub fn without_tokens(self) -> WithoutTokens<Self> {
        WithoutTokens::new(self)
    }

    /// Get the next node including the depth which it is located at. This
    /// exists as an alternative to coercing this iterator into [WithDepths].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "c1" => {
    ///             "c2",
    ///             "c3",
    ///         }
    ///     }
    /// };
    ///
    /// let mut it = tree.walk();
    ///
    /// assert_eq!(it.next_with_depth().map(|(d, n)| (d, *n.value())), Some((0, "root")));
    /// assert_eq!(it.next_with_depth().map(|(d, n)| (d, *n.value())), Some((1, "c1")));
    /// assert_eq!(it.next().map(|n| *n.value()), Some("c2"));
    /// assert_eq!(it.next_with_depth().map(|(d, n)| (d, *n.value())), Some((2, "c3")));
    /// assert_eq!(it.next(), None);
    /// # Ok(()) }
    /// ```
    pub fn next_with_depth(&mut self) -> Option<(usize, Node<'a, T>)> {
        loop {
            let depth = self.iter.depth();
            let (e, node) = self.iter.next()?;

            if !matches!(e, Event::Up) {
                return Some((depth, node));
            }
        }
    }
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (e, node) = self.iter.next()?;

            if !matches!(e, Event::Up) {
                return Some(node);
            }
        }
    }
}

/// An iterator that walks over the entire tree, visiting every node exactly
/// once. This is constructed with [Walk::with_depths].
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// assert_eq!(
///     tree.walk().with_depths().map(|(d, n)| (d, *n.value())).collect::<Vec<_>>(),
///     [(0, "root"), (1, "c1"), (2, "c2"), (2, "c3"), (2, "c4"), (1, "c5"), (1, "c6")]
/// );
///
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.walk().with_depths().map(|(d, n)| (d, *n.value())).collect::<Vec<_>>(),
///     [(0, "c1"), (1, "c2"), (1, "c3"), (1, "c4"), (0, "c5"), (0, "c6")]
/// );
/// # Ok(()) }
/// ```
pub struct WithDepths<'a, T> {
    iter: Walk<'a, T>,
}

impl<'a, T> Iterator for WithDepths<'a, T> {
    type Item = (usize, Node<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_with_depth()
    }
}
