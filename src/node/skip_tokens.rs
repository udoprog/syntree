use core::iter::FusedIterator;

use crate::node::Node;
use crate::pointer::Width;

/// Wrapped around an iterator that excludes nodes without children.
///
/// See [`Siblings::skip_tokens`] or [`Walk::skip_tokens`].
///
/// [`Siblings::skip_tokens`]: crate::node::Siblings::skip_tokens
/// [`Walk::skip_tokens`]: crate::node::Walk::skip_tokens
///
/// # Examples
///
/// Filtering childless nodes from a [`Siblings`] iterator:
///
/// ```
/// let tree = syntree::tree! {
///     ("token1", 1),
///     "child1" => {
///         "token2"
///     },
///     ("token3", 1),
///     "child2" => {
///         "toke4"
///     },
///     ("token5", 1),
///     "child3" => {
///         "token6"
///     },
///     ("token7", 1)
/// };
///
/// let mut it = tree.children().skip_tokens();
///
/// assert_eq!(
///     it.map(|n| *n.value()).collect::<Vec<_>>(),
///     ["child1", "child2", "child3"]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// Filtering tokens from a [`Walk`] iterator:
///
/// ```
/// let tree = syntree::tree! {
///     "child1" => {
///         "child2" => {
///             "token1"
///         },
///         ("token2", 1),
///         "child3" => {
///             "token3"
///         },
///     },
///     "child4" => {
///         ("token4", 1)
///     }
/// };
///
/// let mut it = tree.walk().skip_tokens();
///
/// assert_eq!(
///     it.map(|n| *n.value()).collect::<Vec<_>>(),
///     ["child1", "child2", "child3", "child4"]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// [`Siblings`]: crate::node::Siblings
/// [`Walk`]: crate::node::Walk
pub struct SkipTokens<U> {
    iter: U,
}

impl<U> SkipTokens<U> {
    #[inline]
    pub(crate) const fn new(iter: U) -> Self {
        Self { iter }
    }
}

impl<'a, U, T: 'a, I: 'a, W: 'a> Iterator for SkipTokens<U>
where
    W: Width,
    U: Iterator<Item = Node<'a, T, I, W>>,
{
    type Item = U::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|n| n.has_children())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }

    #[inline]
    fn find<F>(&mut self, mut predicate: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        self.iter.find(move |n| n.has_children() && predicate(n))
    }
}

impl<'a, U, T: 'a, I: 'a, W: 'a> DoubleEndedIterator for SkipTokens<U>
where
    W: Width,
    U: DoubleEndedIterator<Item = Node<'a, T, I, W>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.rfind(|n| n.has_children())
    }

    #[inline]
    fn rfind<F>(&mut self, mut predicate: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        self.iter.rfind(move |n| n.has_children() && predicate(n))
    }
}

impl<'a, U, T: 'a, I: 'a, W: 'a> FusedIterator for SkipTokens<U>
where
    W: Width,
    U: FusedIterator<Item = Node<'a, T, I, W>>,
{
}

impl<U> Clone for SkipTokens<U>
where
    U: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<U> Default for SkipTokens<U>
where
    U: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: Default::default(),
        }
    }
}
