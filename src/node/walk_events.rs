use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::links::Links;
use crate::node::Node;
use crate::pointer::Pointer;

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
///     tree.walk_events().map(|(e, n)| (e, n.value())).collect::<Vec<_>>(),
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
///     root.walk_events().map(|(e, n)| (e, n.value())).collect::<Vec<_>>(),
///     [
///         (Next, "root"),
///         (Down, "c1"),
///         (Down, "c2"),
///         (Next, "c3"),
///         (Next, "c4"),
///         (Up, "c1"),
///         (Next, "c5"),
///         (Next, "c6"),
///         (Up, "root"),
///     ]
/// );
///
/// let c1 = root.first().ok_or("missing c1")?;
///
/// assert_eq!(
///     c1.walk_events().map(|(e, n)| (e, n.value())).collect::<Vec<_>>(),
///     [
///         (Next, "c1"),
///         (Down, "c2"),
///         (Next, "c3"),
///         (Next, "c4"),
///         (Up, "c1"),
///         (Next, "c5"),
///         (Next, "c6"),
///         (Up, "root"),
///     ]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
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
///             nodes.push((depth, node.value()));
///         }
///         Next => {
///             nodes.push((depth, node.value()));
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
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct WalkEvents<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// The tree being iterated over.
    tree: &'a [Links<T, F::Index, F::Pointer>],
    // The current node.
    node: Option<(F::Pointer, Event)>,
    // Current depth being walked.
    depth: isize,
}

impl<'a, T, F> WalkEvents<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new events walker.
    #[inline]
    pub(crate) fn new(
        tree: &'a [Links<T, F::Index, F::Pointer>],
        node: Option<F::Pointer>,
        e: Event,
    ) -> Self {
        Self {
            tree,
            node: node.map(|n| (n, e)),
            depth: 0,
        }
    }

    /// Get the depth of the node that the next call to [`next`] returns,
    /// relative to the node the walk started at, which has depth `0`.
    ///
    /// A node emitted with [`Event::Up`] has the depth of the node being
    /// returned to. When a walk started at a node leaves that node's subtree,
    /// for example by climbing up to its parent, the depth becomes negative.
    ///
    /// [`next`]: Iterator::next
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
    ///         },
    ///         "c3" => {}
    ///     }
    /// };
    ///
    /// let mut it = tree.walk_events();
    /// let mut out = Vec::new();
    ///
    /// loop {
    ///     let depth = it.depth();
    ///     let Some((event, node)) = it.next() else { break };
    ///     out.push((depth, event, node.value()));
    /// }
    ///
    /// assert_eq!(
    ///     out,
    ///     [
    ///         (0, Next, "root"),
    ///         (1, Down, "c1"),
    ///         (2, Down, "c2"),
    ///         (1, Up, "c1"),
    ///         (1, Next, "c3"),
    ///         (0, Up, "root"),
    ///     ]
    /// );
    ///
    /// let c1 = tree.first().and_then(|n| n.first()).ok_or("missing c1")?;
    ///
    /// let mut it = c1.walk_events();
    /// let mut out = Vec::new();
    ///
    /// loop {
    ///     let depth = it.depth();
    ///     let Some((event, node)) = it.next() else { break };
    ///     out.push((depth, event, node.value()));
    /// }
    ///
    /// assert_eq!(
    ///     out,
    ///     [
    ///         (0, Next, "c1"),
    ///         (1, Down, "c2"),
    ///         (0, Up, "c1"),
    ///         (0, Next, "c3"),
    ///         (-1, Up, "root"),
    ///     ]
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> isize {
        self.depth
    }

    fn step(
        &mut self,
        links: &Links<T, F::Index, F::Pointer>,
        event: Event,
    ) -> Option<(F::Pointer, Event)> {
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

impl<T, F> Clone for WalkEvents<'_, T, F>
where
    T: Copy,
    F: Flavor,
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

impl<T, F> Default for WalkEvents<'_, T, F>
where
    T: Copy,
    F: Flavor,
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

impl<'a, T, F> Iterator for WalkEvents<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = (Event, Node<'a, T, F>);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, event) = self.node.take()?;
        let links = self.tree.get(node.get())?;
        self.node = self.step(links, event);
        let node = Node::new(links, self.tree);
        Some((event, node))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::from(self.node.is_some()), None)
    }
}

impl<T, F> FusedIterator for WalkEvents<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> fmt::Debug for WalkEvents<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let next = self.node.and_then(|(id, event)| {
            let links = self.tree.get(id.get())?;
            Some((event, Node::<T, F>::new(links, self.tree)))
        });

        f.debug_struct("WalkEvents")
            .field("next", &next)
            .field("depth", &self.depth)
            .finish()
    }
}
