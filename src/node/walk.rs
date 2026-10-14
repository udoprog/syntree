use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::links::Links;
use crate::node::Node;
use crate::node::{Event, SkipTokens, WalkEvents};

/// An iterator that walks over the entire tree, visiting every node exactly
/// once.
///
/// See [`Tree::walk`][crate::Tree::walk] or [`Node::walk`].
///
/// # Examples
///
/// ```
/// let tree = syntree::tree! {
///     "root" => {
///         "c1" => {
///             "c2" => {},
///             "c3" => {},
///             "c4" => {},
///         },
///         "c5" => {},
///         "c6" => {
///            "c7" => {},
///         }
///     }
/// };
///
/// // Walk the entire tree.
/// assert_eq!(
///     tree.walk().map(|n| n.value()).collect::<Vec<_>>(),
///     ["root", "c1", "c2", "c3", "c4", "c5", "c6", "c7"],
/// );
///
/// // Walk from the root.
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.walk().map(|n| n.value()).collect::<Vec<_>>(),
///     ["root", "c1", "c2", "c3", "c4", "c5", "c6", "c7"],
/// );
///
/// // Walk from c1 and visit siblings.
/// let c1 = root.first().ok_or("missing c1 node")?;
///
/// assert_eq!(
///     c1.walk().map(|n| n.value()).collect::<Vec<_>>(),
///     ["c1", "c2", "c3", "c4", "c5", "c6", "c7"],
/// );
///
/// assert_eq!(
///     c1.walk_from().map(|n| n.value()).collect::<Vec<_>>(),
///     ["c5", "c6", "c7"],
/// );
///
/// // Walk from c4 and visit parent siblings.
/// let c4 = c1.last().ok_or("missing c1 node")?;
///
/// assert_eq!(
///     c4.walk().map(|n| n.value()).collect::<Vec<_>>(),
///     ["c4", "c5", "c6", "c7"],
/// );
///
/// assert_eq!(
///     c4.walk_from().map(|n| n.value()).collect::<Vec<_>>(),
///     ["c5", "c6", "c7"],
/// );
///
/// // Walk from c5 and visit siblings.
/// let c5 = c1.next().ok_or("missing c5 node")?;
///
/// assert_eq!(
///     c5.walk().map(|n| n.value()).collect::<Vec<_>>(),
///     ["c5", "c6", "c7"],
/// );
///
/// assert_eq!(
///     c5.walk_from().map(|n| n.value()).collect::<Vec<_>>(),
///     ["c6", "c7"],
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    iter: WalkEvents<'a, T, F>,
    // A tree-level walk has no single node to stay inside, so `inside` must
    // not stop when the walk climbs back to the top level.
    bounded: bool,
}

impl<'a, T, F> Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new walk starting at a node.
    #[inline]
    pub(crate) fn new(
        tree: &'a [Links<T, F::Index, F::Pointer>],
        node: Option<F::Pointer>,
        e: Event,
    ) -> Self {
        Self {
            iter: WalkEvents::new(tree, node, e),
            bounded: true,
        }
    }

    /// Construct a new walk over every root of a tree.
    #[inline]
    pub(crate) fn tree(
        tree: &'a [Links<T, F::Index, F::Pointer>],
        node: Option<F::Pointer>,
    ) -> Self {
        Self {
            iter: WalkEvents::new(tree, node, Event::Next),
            bounded: false,
        }
    }

    /// Convert this iterator into one that limits the walk to inside the
    /// node it started at, visiting every node exactly once.
    ///
    /// The scope is the node the walk was constructed from, such as through
    /// [`Node::walk`] or [`Node::walk_from`], even if this iterator has already
    /// been advanced. A walk constructed through
    /// [`Tree::walk`][crate::Tree::walk] has the whole tree as its scope, so it
    /// visits the remaining nodes under every root.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "n1" => {
    ///         "n3" => {
    ///             "n4" => {},
    ///             "n5" => {},
    ///         }
    ///     },
    ///     "n6" => {
    ///         "n7"
    ///     }
    /// };
    ///
    /// let values = tree.walk().map(|n| n.value()).collect::<Vec<_>>();
    /// assert_eq!(values, ["n1", "n3", "n4", "n5", "n6", "n7"]);
    ///
    /// let n1 = tree.first().ok_or("missing n1")?;
    ///
    /// let values = n1.walk().map(|n| n.value()).collect::<Vec<_>>();
    /// assert_eq!(values, ["n1", "n3", "n4", "n5", "n6", "n7"]);
    ///
    /// let values = n1.walk().inside().map(|n| n.value()).collect::<Vec<_>>();
    /// assert_eq!(values, ["n1", "n3", "n4", "n5"]);
    ///
    /// let values = n1.walk_from().inside().map(|n| n.value()).collect::<Vec<_>>();
    /// let empty: [&str; 0] = [];
    /// assert_eq!(values, empty);
    ///
    /// let values = tree.walk().inside().map(|n| n.value()).collect::<Vec<_>>();
    /// assert_eq!(values, ["n1", "n3", "n4", "n5", "n6", "n7"]);
    ///
    /// let mut walk = n1.walk();
    /// assert_eq!(walk.next().map(|n| n.value()), Some("n1"));
    /// assert_eq!(walk.next().map(|n| n.value()), Some("n3"));
    /// let values = walk.inside().map(|n| n.value()).collect::<Vec<_>>();
    /// assert_eq!(values, ["n4", "n5"]);
    ///
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn inside(self) -> Inside<'a, T, F> {
        Inside {
            iter: self.iter,
            bounded: self.bounded,
        }
    }

    /// Convert this iterator into one which includes depths.
    ///
    /// See [`WithDepths`] for how depths are counted.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "c1" => {
    ///             "c2" => {},
    ///             "c3" => {},
    ///         }
    ///     }
    /// };
    ///
    /// let mut it = tree.walk().with_depths().map(|(d, n)| (d, n.value()));
    /// assert!(it.eq([(0, "root"), (1, "c1"), (2, "c2"), (2, "c3")]));
    ///
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn with_depths(self) -> WithDepths<'a, T, F> {
        WithDepths { iter: self.iter }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this iterator.
    /// This filters out childless nodes, also known as tokens.
    ///
    /// See [`SkipTokens`] for documentation.
    #[inline]
    #[must_use]
    pub fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }

    /// Get the next node with a corresponding depth.
    ///
    /// Depths are relative to the node the walk started at and become negative
    /// once a walk started from a [`Node`] leaves that node's subtree, see
    /// [`WithDepths`].
    ///
    /// Alternatively you can use [`WithDepths`] through [`Walk::with_depths`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter;
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
    /// let mut it = tree.walk();
    /// let it = iter::from_fn(move || it.next_with_depth());
    /// let it = it.map(|(d, n)| (d, n.value()));
    ///
    /// assert!(it.eq([(0, "root"), (1, "c1"), (2, "c2"), (2, "c3")]));
    ///
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn next_with_depth(&mut self) -> Option<(isize, Node<'a, T, F>)> {
        next_with_depth(&mut self.iter)
    }
}

impl<T, F> Clone for Walk<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            bounded: self.bounded,
        }
    }
}

impl<T, F> Default for Walk<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
            bounded: true,
        }
    }
}

impl<'a, T, F> Iterator for Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Node<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (event, node) = self.iter.next()?;

            if !matches!(event, Event::Up) {
                return Some(node);
            }
        }
    }
}

impl<T, F> FusedIterator for Walk<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

/// An iterator that walks over the entire tree, visiting every node exactly
/// once and yielding each node together with its depth. This is constructed
/// with [`Walk::with_depths`].
///
/// Depths are relative to the node the walk started at, which has depth `0`.
/// A walk started from a [`Node`] continues past the end of that node's
/// subtree into the following siblings of its ancestors, and those nodes have
/// negative depths. This is why depths are [`isize`]. A walk started through
/// [`Tree::walk`][crate::Tree::walk] gives every root depth `0`.
///
/// # Examples
///
/// ```
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
///     tree.walk().with_depths().map(|(d, n)| (d, n.value())).collect::<Vec<_>>(),
///     [
///         (0, "root"),
///         (1, "c1"),
///         (2, "c2"),
///         (2, "c3"),
///         (2, "c4"),
///         (1, "c5"),
///         (1, "c6")
///     ]
/// );
///
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.walk().with_depths().map(|(d, n)| (d, n.value())).collect::<Vec<_>>(),
///     [
///         (0, "root"),
///         (1, "c1"),
///         (2, "c2"),
///         (2, "c3"),
///         (2, "c4"),
///         (1, "c5"),
///         (1, "c6")
///     ]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// Leaving the subtree of the starting node gives negative depths:
///
/// ```
/// let tree = syntree::tree! {
///     "root" => {
///         "b" => {
///             "b1" => {}
///         },
///         "c" => {},
///         "d" => {}
///     },
///     "root2" => {
///         "e" => {}
///     }
/// };
///
/// let b1 = tree.walk().find(|n| n.value() == "b1").ok_or("missing b1")?;
///
/// assert_eq!(
///     b1.walk().with_depths().map(|(d, n)| (d, n.value())).collect::<Vec<_>>(),
///     [(0, "b1"), (-1, "c"), (-1, "d"), (-2, "root2"), (-1, "e")]
/// );
///
/// assert_eq!(
///     tree.walk().with_depths().map(|(d, n)| (d, n.value())).collect::<Vec<_>>(),
///     [(0, "root"), (1, "b"), (2, "b1"), (1, "c"), (1, "d"), (0, "root2"), (1, "e")]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct WithDepths<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    iter: WalkEvents<'a, T, F>,
}

impl<'a, T, F> Iterator for WithDepths<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = (isize, Node<'a, T, F>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        next_with_depth(&mut self.iter)
    }
}

#[inline]
fn next_with_depth<'a, T, F>(iter: &mut WalkEvents<'a, T, F>) -> Option<(isize, Node<'a, T, F>)>
where
    T: Copy,
    F: Flavor,
{
    loop {
        let depth = iter.depth();
        let (event, node) = iter.next()?;

        if !matches!(event, Event::Up) {
            return Some((depth, node));
        }
    }
}

impl<T, F> FusedIterator for WithDepths<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for WithDepths<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, F> Default for WithDepths<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
        }
    }
}

/// An iterator that limits the walk to inside the node it started at, visiting
/// every node exactly once. This is constructed with [`Walk::inside`].
///
/// The scope is the node the [`Walk`] was constructed from, even if it was
/// advanced before calling [`Walk::inside`]. For a walk constructed through
/// [`Tree::walk`][crate::Tree::walk] the scope is the whole tree, covering
/// every root.
///
/// # Examples
///
/// ```
/// let tree = syntree::tree! {
///     "n1" => {
///         "n3" => {
///             "n4" => {},
///             "n5" => {},
///         }
///     },
///     "n6" => {
///         "n7"
///     }
/// };
///
/// let values = tree.walk().map(|n| n.value()).collect::<Vec<_>>();
/// assert_eq!(values, ["n1", "n3", "n4", "n5", "n6", "n7"]);
///
/// let n1 = tree.first().ok_or("missing n1")?;
///
/// let values = n1.walk().map(|n| n.value()).collect::<Vec<_>>();
/// assert_eq!(values, ["n1", "n3", "n4", "n5", "n6", "n7"]);
///
/// let values = n1.walk().inside().map(|n| n.value()).collect::<Vec<_>>();
/// assert_eq!(values, ["n1", "n3", "n4", "n5"]);
///
/// let values = n1.walk_from().inside().map(|n| n.value()).collect::<Vec<_>>();
/// let empty: [&str; 0] = [];
/// assert_eq!(values, empty);
///
/// let values = tree.walk().inside().map(|n| n.value()).collect::<Vec<_>>();
/// assert_eq!(values, ["n1", "n3", "n4", "n5", "n6", "n7"]);
///
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// The iterator can be named, for example to store it in a struct:
///
/// ```
/// use syntree::node::Inside;
/// use syntree::FlavorDefault;
///
/// struct Values<'a> {
///     iter: Inside<'a, &'static str, FlavorDefault>,
/// }
///
/// impl Iterator for Values<'_> {
///     type Item = &'static str;
///
///     fn next(&mut self) -> Option<Self::Item> {
///         Some(self.iter.next()?.value())
///     }
/// }
///
/// let tree = syntree::tree! {
///     "n1" => {
///         "n2" => {}
///     },
///     "n3" => {}
/// };
///
/// let n1 = tree.first().ok_or("missing n1")?;
/// let values = Values { iter: n1.walk().inside() };
/// assert_eq!(values.collect::<Vec<_>>(), ["n1", "n2"]);
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Inside<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    iter: WalkEvents<'a, T, F>,
    bounded: bool,
}

impl<'a, T, F> Iterator for Inside<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Node<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (event, node) = self.iter.next()?;

            if self.bounded && self.iter.depth() <= 0 {
                self.iter = WalkEvents::default();
            }

            if !matches!(event, Event::Up) {
                return Some(node);
            }
        }
    }
}

impl<T, F> FusedIterator for Inside<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for Inside<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            bounded: self.bounded,
        }
    }
}

impl<T, F> Default for Inside<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
            bounded: true,
        }
    }
}

impl<T, F> fmt::Debug for Walk<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Walk")
            .field("events", &self.iter)
            .finish_non_exhaustive()
    }
}

impl<T, F> fmt::Debug for WithDepths<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WithDepths")
            .field("events", &self.iter)
            .finish()
    }
}

impl<T, F> fmt::Debug for Inside<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inside")
            .field("events", &self.iter)
            .finish_non_exhaustive()
    }
}
