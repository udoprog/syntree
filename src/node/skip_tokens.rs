use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::node::{Element, Node};

/// Wrapped around an iterator that excludes tokens, yielding only interior
/// [`Node`]s.
///
/// Interior nodes are kept even when they are empty, since they were opened
/// as nodes.
///
/// See [`Children::skip_tokens`], [`Siblings::skip_tokens`],
/// [`Ancestors::skip_tokens`] or [`Walk::skip_tokens`].
///
/// [`Children::skip_tokens`]: crate::node::Children::skip_tokens
/// [`Siblings::skip_tokens`]: crate::node::Siblings::skip_tokens
/// [`Ancestors::skip_tokens`]: crate::node::Ancestors::skip_tokens
/// [`Walk::skip_tokens`]: crate::node::Walk::skip_tokens
///
/// # Examples
///
/// Filtering tokens from a [`Children`] iterator:
///
/// ```
/// let tree = syntree::tree! {
///     ("token1", 1),
///     "child1" => {
///         "token2"
///     },
///     ("token3", 1),
///     "child2" => {
///         "token4"
///     },
///     ("token5", 1),
///     "child3" => {
///         "token6"
///     },
///     ("token7", 1)
/// };
///
/// let it = tree.children().skip_tokens();
///
/// assert_eq!(
///     it.map(|n| n.value()).collect::<Vec<_>>(),
///     ["child1", "child2", "child3"]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
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
/// let it = tree.walk().skip_tokens();
///
/// assert_eq!(
///     it.map(|n| n.value()).collect::<Vec<_>>(),
///     ["child1", "child2", "child3", "child4"]
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// [`Children`]: crate::node::Children
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

impl<'a, U, T: 'a, F: 'a> Iterator for SkipTokens<U>
where
    T: Copy,
    F: Flavor,
    U: Iterator<Item = Element<'a, T, F>>,
{
    type Item = Node<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find_map(Element::into_node)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.iter
            .find_map(move |e| e.into_node().filter(|n| predicate(n)))
    }
}

impl<'a, U, T: 'a, F: 'a> DoubleEndedIterator for SkipTokens<U>
where
    T: Copy,
    F: Flavor,
    U: DoubleEndedIterator<Item = Element<'a, T, F>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().rev().find_map(Element::into_node)
    }

    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.iter
            .by_ref()
            .rev()
            .find_map(move |e| e.into_node().filter(|n| predicate(n)))
    }
}

impl<'a, U, T: 'a, F: 'a> FusedIterator for SkipTokens<U>
where
    T: Copy,
    F: Flavor,
    U: FusedIterator<Item = Element<'a, T, F>>,
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

impl<U> fmt::Debug for SkipTokens<U>
where
    U: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SkipTokens")
            .field("iter", &self.iter)
            .finish()
    }
}
