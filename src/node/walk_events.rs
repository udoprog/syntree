use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::node::{Element, Id};
use crate::pointer::Pointer;
use crate::tree::Tree;

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

/// The next element of a walk, how it is reached, and the next interior node
/// at its level.
type State<P> = (Id<P>, Event, Option<P>);

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
    tree: Option<&'a Tree<T, F>>,
    /// The next element, how it is reached, and the next interior node at its
    /// level, which is only used when the element is a token.
    node: Option<State<F::Pointer>>,
    /// Current depth being walked.
    depth: isize,
    /// Whether trivia tokens are yielded.
    trivia: bool,
}

impl<'a, T, F> WalkEvents<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Walk from `start`, where `next` is the next interior node at the level
    /// of `start` if it is a token.
    ///
    /// The first element is reached with `event`. With [`Event::Up`] the walk
    /// does not descend into `start`.
    #[inline]
    pub(crate) fn new(
        tree: &'a Tree<T, F>,
        start: Option<Id<F::Pointer>>,
        next: Option<F::Pointer>,
        event: Event,
    ) -> Self {
        Self {
            tree: Some(tree),
            node: start.map(|id| (id, event, next)),
            depth: 0,
            trivia: false,
        }
    }

    /// Include trivia tokens, which this iterator skips by default.
    ///
    /// Skipping trivia keeps the events balanced: a node whose only children
    /// are trivia is walked like a node without children, so it gets neither
    /// a [`Event::Down`] into it nor an [`Event::Up`] out of it.
    ///
    /// See [`Builder::trivia`][crate::Builder::trivia].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::node::Event::*;
    ///
    /// let mut tree = syntree::Builder::new();
    /// tree.open("a")?;
    /// tree.trivia("ws", 1)?;
    /// tree.close()?;
    /// tree.open("b")?;
    /// tree.trivia("ws", 1)?;
    /// tree.token("lit", 1)?;
    /// tree.close()?;
    /// let tree = tree.build()?;
    ///
    /// let events = tree.walk_events().map(|(e, n)| (e, n.value())).collect::<Vec<_>>();
    /// assert_eq!(events, [(Next, "a"), (Next, "b"), (Down, "lit"), (Up, "b")]);
    ///
    /// let events = tree
    ///     .walk_events()
    ///     .with_trivia()
    ///     .map(|(e, n)| (e, n.value()))
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     events,
    ///     [
    ///         (Next, "a"),
    ///         (Down, "ws"),
    ///         (Up, "a"),
    ///         (Next, "b"),
    ///         (Down, "ws"),
    ///         (Next, "lit"),
    ///         (Up, "b"),
    ///     ]
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_trivia(mut self) -> Self {
        self.trivia = true;
        self
    }

    /// The next event with the depth it is at, skipping trivia unless it is
    /// included. With `inside` the walk ends once it is back at depth zero or
    /// above, which is what [`Walk::inside`][crate::node::Walk::inside]
    /// does.
    pub(crate) fn next_with(&mut self, inside: bool) -> Option<(isize, Event, Element<'a, T, F>)> {
        let tree = self.tree?;
        // A `Down` into a skipped trivia token, which the next element at its
        // level takes over.
        let mut down = false;

        loop {
            let depth = self.depth;
            let (id, mut event, next) = self.node.take()?;
            self.node = self.step(tree, id, event, next);

            if inside && self.depth <= 0 {
                self.node = None;
            }

            if !self.trivia && tree.is_trivia(id) {
                down |= matches!(event, Event::Down);
                continue;
            }

            if down {
                down = false;

                match event {
                    Event::Next => event = Event::Down,
                    // Every child was trivia, so the node was never entered.
                    Event::Up => continue,
                    Event::Down => {}
                }
            }

            return Some((depth, event, tree.element(id)?));
        }
    }

    /// Get the depth of the node that the next call to [`next`] returns,
    /// relative to the node the walk started at, which has depth `0`.
    ///
    /// A node emitted with [`Event::Up`] has the depth of the node being
    /// returned to. When a walk started at a node leaves that node's subtree,
    /// for example by climbing up to its parent, the depth becomes negative.
    ///
    /// Once the iterator is exhausted, the depth is that of the last node it
    /// returned.
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
        tree: &'a Tree<T, F>,
        id: Id<F::Pointer>,
        event: Event,
        next: Option<F::Pointer>,
    ) -> Option<State<F::Pointer>> {
        let parent = match id {
            Id::Node(n) => {
                let links = tree.node_links(n)?;

                if !matches!(event, Event::Up) {
                    if let Some(first) = tree.next_at(Some(n), links.tok_start.get(), links.first) {
                        self.depth = self.depth.checked_add(1)?;
                        return Some((first, Event::Down, links.first));
                    }
                }

                if let Some(id) = tree.next_at(links.parent, links.tok_end.get(), links.next) {
                    return Some((id, Event::Next, links.next));
                }

                links.parent
            }
            Id::Token(t) => {
                let index = t.get();
                let links = tree.tokens().get(index)?;

                if let Some(id) = tree.next_at(links.parent(), index.checked_add(1)?, next) {
                    return Some((id, Event::Next, next));
                }

                links.parent()
            }
        };

        let parent = parent?;
        self.depth = self.depth.checked_sub(1)?;
        Some((Id::Node(parent), Event::Up, None))
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
            trivia: self.trivia,
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
            tree: None,
            node: None,
            depth: 0,
            trivia: false,
        }
    }
}

impl<'a, T, F> Iterator for WalkEvents<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = (Event, Element<'a, T, F>);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, event, element) = self.next_with(false)?;
        Some((event, element))
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
        let next = self
            .tree
            .zip(self.node)
            .and_then(|(tree, (id, event, _))| Some((event, tree.element(id)?)));

        f.debug_struct("WalkEvents")
            .field("next", &next)
            .field("depth", &self.depth)
            .finish()
    }
}
