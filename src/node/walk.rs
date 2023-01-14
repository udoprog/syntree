use std::iter::FusedIterator;

use crate::links::Links;
use crate::node::{Event, SkipTokens, WalkEvents};
use crate::non_max::NonMax;
use crate::Node;

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
///         "c6" => {}
///     }
/// };
///
/// // Walk the entire tree.
/// assert!(
///     tree.walk().map(|n| *n.value()).eq(["root", "c1", "c2", "c3", "c4", "c5", "c6"])
/// );
///
/// // Walk from the root.
/// let root = tree.first().ok_or("missing root node")?;
/// assert!(
///     root.walk().map(|n| *n.value()).eq(["c1", "c2", "c3", "c4", "c5", "c6"])
/// );
///
/// // Walk from second child of the root. Note that the node itself is correctly excluded.
/// let c5 = root.first().and_then(|n| n.next()).ok_or("missing second child")?;
/// assert_eq!(c5.walk().map(|n| *n.value()).collect::<Vec<_>>(), Vec::<&str>::new());
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct Walk<'a, T, S> {
    iter: WalkEvents<'a, T, S>,
}

impl<'a, T, S> Walk<'a, T, S> {
    /// Construct a new walk.
    #[inline]
    pub(crate) const fn new(tree: &'a [Links<T, S>], node: Option<NonMax>) -> Self {
        Self {
            iter: WalkEvents::new(tree, node),
        }
    }

    /// Get the next element with a corresponding depth.
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
    pub fn next_with_depth(&mut self) -> Option<(usize, Node<'a, T, S>)> {
        loop {
            let depth = self.iter.depth();
            let (event, node) = self.iter.next()?;

            if !matches!(event, Event::Up) {
                return Some((depth, node));
            }
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
    pub fn with_depths(self) -> WithDepths<'a, T, S> {
        WithDepths { iter: self }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this
    /// iterator. This filters out [`Kind::Token`][crate::Kind::Token] elements.
    ///
    /// See [`SkipTokens`] for documentation.
    #[inline]
    #[must_use]
    pub fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }
}

impl<T, S> Clone for Walk<'_, T, S> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S> Default for Walk<'_, T, S> {
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
        }
    }
}

impl<'a, T, S> Iterator for Walk<'a, T, S> {
    type Item = Node<'a, T, S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (e, node) = self.iter.next()?;

            if !matches!(e, Event::Up) {
                return Some(node);
            }
        }
    }
}

impl<T, S> FusedIterator for Walk<'_, T, S> {}

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
///     [(0, "root"), (1, "c1"), (2, "c2"), (2, "c3"), (2, "c4"), (1, "c5"), (1, "c6")]
/// );
///
/// let root = tree.first().ok_or("missing root node")?;
///
/// assert_eq!(
///     root.walk().with_depths().map(|(d, n)| (d, *n.value())).collect::<Vec<_>>(),
///     [(0, "c1"), (1, "c2"), (1, "c3"), (1, "c4"), (0, "c5"), (0, "c6")]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct WithDepths<'a, T, S> {
    iter: Walk<'a, T, S>,
}

impl<'a, T, S> Iterator for WithDepths<'a, T, S> {
    type Item = (usize, Node<'a, T, S>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_with_depth()
    }
}

impl<T, S> FusedIterator for WithDepths<'_, T, S> {}

impl<T, S> Clone for WithDepths<'_, T, S> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S> Default for WithDepths<'_, T, S> {
    #[inline]
    fn default() -> Self {
        Self {
            iter: Walk::default(),
        }
    }
}
