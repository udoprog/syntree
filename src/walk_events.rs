use crate::Node;

/// An event indicating how a tree is being walked with [WalkEvents].
///
/// See [WalkEvents] for documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    /// Walk the next sibling node. This is also the initial event being emitted
    /// when entering the iterator.
    Next,
    /// Walk down the first child of a sub tree.
    Down,
    /// Walk up a single step from a sub tree.
    Up,
}

/// A low-level iterator which walks the tree while emitting [Event] instances
/// indicating *how* the structure is being navigated.
///
/// See [Tree::walk_events][crate::Tree::walk_events] or
/// [Node::walk_events][crate::Node::walk_events].
///
/// # Examples
///
/// ```
/// use syntree::Event::*;
///
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
///     tree.walk_events().map(|(e, n)| (e, *n.value())).collect::<Vec<_>>(),
///     [
///         (Next, "root"),
///         (Down, "c1"),
///         (Down, "c2"),
///         (Next, "c3"),
///         (Next, "c4"),
///         (Up, "c1"),
///         (Next, "c5"),
///         (Next, "c6"),
///         (Up, "root")
///     ]
/// );
///
/// let root = tree.first().ok_or("missing root")?;
///
/// assert_eq!(
///     root.walk_events().map(|(e, n)| (e, *n.value())).collect::<Vec<_>>(),
///     [
///         (Next, "c1"),
///         (Down, "c2"),
///         (Next, "c3"),
///         (Next, "c4"),
///         (Up, "c1"),
///         (Next, "c5"),
///         (Next, "c6")
///     ]
/// );
///
/// let c1 = root.first().ok_or("missing c1")?;
///
/// assert_eq!(
///     c1.walk_events().map(|(e, n)| (e, *n.value())).collect::<Vec<_>>(),
///     [(Next, "c2"), (Next, "c3"), (Next, "c4")]
/// );
/// # Ok(()) }
/// ```
pub struct WalkEvents<'a, T> {
    // The current node.
    start: Option<(Node<'a, T>, Event)>,
    // Parent nodes.
    parents: Vec<Node<'a, T>>,
}

impl<'a, T> WalkEvents<'a, T> {
    /// Construct a new events walker.
    pub(crate) const fn new(node: Option<Node<'a, T>>) -> Self {
        Self {
            start: match node {
                Some(n) => Some((n, Event::Next)),
                None => None,
            },
            parents: Vec::new(),
        }
    }

    /// Get the current depth of the iterator.
    ///
    /// # Examples
    ///
    /// Somewhat unintuitively if you want to know the depth of the next element
    /// from the iterator you need to query the depth *before* advancing the
    /// iterator.
    ///
    /// ```
    /// use syntree::Event::*;
    ///
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
    /// let mut it = tree.walk_events();
    /// let mut depth = it.depth();
    ///
    /// let mut nodes = Vec::new();
    ///
    /// while let Some((event, node)) = it.next() {
    ///     // Only register each node once.
    ///     if !matches!(event, Up) {
    ///         nodes.push((depth, *node.value()));
    ///     }
    ///
    ///     // Query the *next* depth here.
    ///     depth = it.depth();
    /// }
    ///
    /// assert_eq!(it.depth(), 0);
    ///
    /// assert_eq!(
    ///     nodes,
    ///     [(0, "root"), (1, "c1"), (2, "c2"), (2, "c3")]
    /// );
    /// # Ok(()) }
    /// ```
    pub fn depth(&self) -> usize {
        self.parents.len()
    }

    fn step(&mut self, node: Node<'a, T>, event: Event) -> Option<(Node<'a, T>, Event)> {
        if matches!(event, Event::Up) {
            if let Some(next) = node.next() {
                return Some((next, Event::Next));
            }
        } else {
            if let Some(first) = node.first() {
                self.parents.push(node);
                return Some((first, Event::Down));
            }

            if let Some(next) = node.next() {
                return Some((next, Event::Next));
            }
        }

        if let Some(parent) = self.parents.pop() {
            return Some((parent, Event::Up));
        }

        None
    }
}

impl<'a, T> Iterator for WalkEvents<'a, T> {
    type Item = (Event, Node<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, event) = self.start.take()?;

        if let Some(id) = self.step(node, event) {
            self.start = Some(id);
        }

        Some((event, node))
    }
}
