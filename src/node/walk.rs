use std::iter::FusedIterator;

use crate::links::Links;
use crate::node::{Event, SkipTokens, WalkEvents};
use crate::pointer::Width;
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
    pub(crate) fn new(tree: &'a [Links<T, I, W::Pointer>], node: Option<W::Pointer>) -> Self {
        Self {
            iter: WalkEvents::new(tree, node),
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

    /// Construct a [`SkipTokens`] iterator from the remainder of this
    /// iterator. This filters out [`Kind::Token`][crate::Kind::Token] elements.
    ///
    /// See [`SkipTokens`] for documentation.
    #[inline]
    #[must_use]
    pub fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
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
    pub fn next_with_depth(&mut self) -> Option<(usize, Node<'a, T, I, W>)> {
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
            let (e, node) = self.iter.next()?;

            if !matches!(e, Event::Up) {
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
    type Item = (usize, Node<'a, T, I, W>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_with_depth()
    }
}

impl<T, I, W> FusedIterator for WithDepths<'_, T, I, W> where W: Width {}

impl<T, I, W> Clone for WithDepths<'_, T, I, W>
where
    W: Width,
    W::Pointer: Copy,
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
