use crate::links::Links;
use crate::non_max::NonMaxUsize;
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
    First,
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
///     tree.walk_events().map(|(e, n)| (e, *n.data())).collect::<Vec<_>>(),
///     [
///         (Next, "root"),
///         (First, "c1"),
///         (First, "c2"),
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
///     root.walk_events().map(|(e, n)| (e, *n.data())).collect::<Vec<_>>(),
///     [
///         (Next, "c1"),
///         (First, "c2"),
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
///     c1.walk_events().map(|(e, n)| (e, *n.data())).collect::<Vec<_>>(),
///     [(Next, "c2"), (Next, "c3"), (Next, "c4")]
/// );
/// # Ok(()) }
/// ```
pub struct WalkEvents<'a, T> {
    tree: &'a [Links<T>],
    // The current node.
    start: Option<(NonMaxUsize, Event)>,
    // Parent nodes.
    parents: Vec<NonMaxUsize>,
}

impl<'a, T> WalkEvents<'a, T> {
    pub(crate) fn new(tree: &'a [Links<T>], start: Option<NonMaxUsize>) -> Self {
        Self {
            tree,
            start: start.map(|id| (id, Event::Next)),
            parents: Vec::new(),
        }
    }

    fn step(
        &mut self,
        id: NonMaxUsize,
        links: &Links<T>,
        event: Event,
    ) -> Option<(NonMaxUsize, Event)> {
        if matches!(event, Event::Up) {
            if let Some(next) = self.tree.get(id.get()).and_then(|links| links.next) {
                return Some((next, Event::Next));
            }
        } else {
            if let Some(first) = links.first {
                self.parents.push(id);
                return Some((first, Event::First));
            }

            if let Some(next) = links.next {
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
        let (id, event) = self.start.take()?;
        let links = self.tree.get(id.get())?;

        if let Some(id) = self.step(id, links, event) {
            self.start = Some(id);
        }

        Some((event, Node::new(links, self.tree)))
    }
}
