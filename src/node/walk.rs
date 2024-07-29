use core::iter::FusedIterator;

use crate::links::Links;
use crate::node::Node;
use crate::node::{Event, SkipTokens, WalkEvents};
use crate::pointer::Width;

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
///     tree.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "c1", "c2", "c3", "c4", "c5", "c6", "c7"],
/// );
///
/// // Walk from the root.
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "c1", "c2", "c3", "c4", "c5", "c6", "c7"],
/// );
///
/// // Walk from c1 and visit siblings.
/// let c1 = root.first().ok_or("missing c1 node")?;
///
/// assert_eq!(
///     c1.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c1", "c2", "c3", "c4", "c5", "c6", "c7"],
/// );
///
/// assert_eq!(
///     c1.walk_up().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c5", "c6", "c7"],
/// );
///
/// // Walk from c4 and visit parent siblings.
/// let c4 = c1.last().ok_or("missing c1 node")?;
///
/// assert_eq!(
///     c4.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c4", "c5", "c6", "c7"],
/// );
///
/// assert_eq!(
///     c4.walk_up().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c5", "c6", "c7"],
/// );
///
/// // Walk from c5 and visit siblings.
/// let c5 = c1.next().ok_or("missing c5 node")?;
///
/// assert_eq!(
///     c5.walk().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c5", "c6", "c7"],
/// );
///
/// assert_eq!(
///     c5.walk_up().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["c6", "c7"],
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct Walk<'a, T, I, W>
where
    W: Width,
{
    iter: WalkEvents<'a, T, I, W>,
}

impl<'a, T, I, W> Walk<'a, T, I, W>
where
    W: Width,
{
    /// Construct a new walk.
    #[inline]
    pub(crate) fn new(
        tree: &'a [Links<T, I, W::Pointer>],
        node: Option<W::Pointer>,
        e: Event,
    ) -> Self {
        Self {
            iter: WalkEvents::new(tree, node, e),
        }
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
    /// let mut it = tree.walk().with_depths().map(|(d, n)| (d, *n.value()));
    /// assert!(it.eq([(0, "root"), (1, "c1"), (2, "c2"), (2, "c3")]));
    ///
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn with_depths(self) -> WithDepths<'a, T, I, W> {
        WithDepths { iter: self }
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
    /// let it = it.map(|(d, n)| (d, *n.value()));
    ///
    /// assert!(it.eq([(0, "root"), (1, "c1"), (2, "c2"), (2, "c3")]));
    ///
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn next_with_depth(&mut self) -> Option<(isize, Node<'a, T, I, W>)> {
        loop {
            let depth = self.iter.depth();
            let (event, node) = self.iter.next()?;

            if !matches!(event, Event::Up) {
                return Some((depth, node));
            }
        }
    }
}

impl<T, I, W> Clone for Walk<'_, T, I, W>
where
    W: Width,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, I, W> Default for Walk<'_, T, I, W>
where
    W: Width,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
        }
    }
}

impl<'a, T, I, W> Iterator for Walk<'a, T, I, W>
where
    W: Width,
{
    type Item = Node<'a, T, I, W>;

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

impl<T, I, W> FusedIterator for Walk<'_, T, I, W> where W: Width {}

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
///     tree.walk().with_depths().map(|(d, n)| (d, *n.value())).collect::<Vec<_>>(),
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
///     root.walk().with_depths().map(|(d, n)| (d, *n.value())).collect::<Vec<_>>(),
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
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct WithDepths<'a, T, I, W>
where
    W: Width,
{
    iter: Walk<'a, T, I, W>,
}

impl<'a, T, I, W> Iterator for WithDepths<'a, T, I, W>
where
    W: Width,
{
    type Item = (isize, Node<'a, T, I, W>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_with_depth()
    }
}

impl<T, I, W> FusedIterator for WithDepths<'_, T, I, W> where W: Width {}

impl<T, I, W> Clone for WithDepths<'_, T, I, W>
where
    W: Width,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, I, W> Default for WithDepths<'_, T, I, W>
where
    W: Width,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: Walk::default(),
        }
    }
}
