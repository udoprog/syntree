use std::iter::FusedIterator;

use crate::links::Links;
use crate::node::Node;
use crate::pointer::{Pointer, Width};

/// An event indicating how a tree is being walked with [`WalkEvents`].
///
/// See [`WalkEvents`] for documentation.
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
/// See [`Tree::walk_events`][crate::Tree::walk_events] or
/// [`Node::walk_events`][crate::Node::walk_events].
///
/// # Examples
///
/// ```
/// use syntree::node::Event::*;
///
/// let tree = syntree::tree! {
///     "root" => {
///         "c1" => {
///             "c2" => {},
///             "c3" => {},
///             "c4" => {},
///         },
///         "c5" => {},
///         "c6" => {}
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
///         (Next, "c6"),
///     ]
/// );
///
/// let c1 = root.first().ok_or("missing c1")?;
///
/// assert_eq!(
///     c1.walk_events().map(|(e, n)| (e, *n.value())).collect::<Vec<_>>(),
///     [(Next, "c2"), (Next, "c3"), (Next, "c4")]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// Example showcasing how we can use events to keep track of the depth in which
/// nodes are being traversed:
///
/// ```
/// use syntree::node::Event::*;
///
/// let tree = syntree::tree! {
///     "root" => {
///         "c1" => {
///             "c2" => {},
///             "c3" => {},
///         }
///     }
/// };
///
/// let mut it = tree.walk_events();
/// let mut depth = 0;
///
/// let mut nodes = Vec::new();
///
/// while let Some((event, node)) = it.next() {
///     // Only register each node once.
///     match event {
///         Up => {
///             depth -= 1;
///         }
///         Down => {
///             depth += 1;
///             nodes.push((depth, *node.value()));
///         }
///         Next => {
///             nodes.push((depth, *node.value()));
///         }
///     }
/// }
///
/// assert_eq!(depth, 0);
///
/// assert_eq!(
///     nodes,
///     [(0, "root"), (1, "c1"), (2, "c2"), (2, "c3")]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct WalkEvents<'a, T, I, W>
where
    W: Width,
{
    /// The tree being iterated over.
    tree: &'a [Links<T, I, W::Pointer>],
    // The current node.
    node: Option<(W::Pointer, Event)>,
    // Current depth being walked.
    depth: usize,
}

impl<'a, T, I, W> WalkEvents<'a, T, I, W>
where
    W: Width,
{
    /// Construct a new events walker.
    #[inline]
    pub(crate) fn new(tree: &'a [Links<T, I, W::Pointer>], node: Option<W::Pointer>) -> Self {
        Self {
            tree,
            node: node.map(|n| (n, Event::Next)),
            depth: 0,
        }
    }

    /// Get current depth.
    pub(crate) fn depth(&self) -> usize {
        self.depth
    }

    fn step(
        &mut self,
        links: &Links<T, I, W::Pointer>,
        event: Event,
    ) -> Option<(W::Pointer, Event)> {
        if let Event::Up = event {
            if let Some(next) = links.next {
                return Some((next, Event::Next));
            }
        } else {
            if let Some(first) = links.first {
                self.depth = self.depth.checked_add(1)?;
                return Some((first, Event::Down));
            }

            if let Some(next) = links.next {
                return Some((next, Event::Next));
            }
        }

        self.depth = self.depth.checked_sub(1)?;
        Some((links.parent?, Event::Up))
    }
}

impl<T, I, W> Clone for WalkEvents<'_, T, I, W>
where
    W: Width,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
            depth: self.depth,
        }
    }
}

impl<T, I, W> Default for WalkEvents<'_, T, I, W>
where
    W: Width,
{
    #[inline]
    fn default() -> Self {
        Self {
            tree: &[],
            node: None,
            depth: 0,
        }
    }
}

impl<'a, T, I, W> Iterator for WalkEvents<'a, T, I, W>
where
    W: Width,
{
    type Item = (Event, Node<'a, T, I, W>);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, event) = self.node.take()?;
        let links = self.tree.get(node.get())?;
        self.node = self.step(links, event);
        let node = Node::new(links, self.tree);
        Some((event, node))
    }
}

impl<T, I, W> FusedIterator for WalkEvents<'_, T, I, W> where W: Width {}
