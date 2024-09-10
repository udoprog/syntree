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
}

impl<'a, T, F> Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new walk.
    #[inline]
    pub(crate) fn new(
        tree: &'a [Links<T, F::Index, F::Pointer>],
        node: Option<F::Pointer>,
        e: Event,
    ) -> Self {
        Self {
            iter: WalkEvents::new(tree, node, e),
        }
    }

    /// Convert this iterator into one that limits the walk to inside the
    /// current node, visiting every node exactly once.
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
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn inside(self) -> Inside<'a, T, F> {
        Inside { iter: self.iter }
    }

    /// Convert this iterator into one which includes depths.
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
        loop {
            let depth = self.iter.depth();
            let (event, node) = self.iter.next()?;

            if !matches!(event, Event::Up) {
                return Some((depth, node));
            }
        }
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
/// once. This is constructed with [`Walk::with_depths`].
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
        loop {
            let depth = self.iter.depth();
            let (event, node) = self.iter.next()?;

            if !matches!(event, Event::Up) {
                return Some((depth, node));
            }
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

/// An iterator that limits the walk to inside the current node, visiting every
/// node exactly once. This is constructed with [`Walk::inside`].
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
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct Inside<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    iter: WalkEvents<'a, T, F>,
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

            if self.iter.depth() <= 0 {
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
        }
    }
}
