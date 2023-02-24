use core::iter::FusedIterator;

use crate::node::Node;
use crate::tree::Kind;

/// Wrapped around an iterator that excludes [`Kind::Token`] nodes.
///
/// See [`Siblings::skip_tokens`] or [`Walk::skip_tokens`].
///
/// [`Siblings::skip_tokens`]: crate::node::Siblings::skip_tokens
/// [`Walk::skip_tokens`]: crate::node::Walk::skip_tokens
///
/// # Examples
///
/// Filtering [`Kind::Token`] elements from a [`Siblings`] iterator:
///
/// ```
/// let tree = syntree::tree! {
///     ("token1", 1),
///     "child1" => {},
///     ("token2", 1),
///     "child2" => {},
///     ("token3", 1),
///     "child3" => {},
///     ("token4", 1)
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
/// Filtering [`Kind::Token`] elements from a [`Walk`] iterator:
///
/// ```
/// let tree = syntree::tree! {
///     "child1" => {
///         "child2" => {},
///         ("token1", 1),
///         "child3" => {},
///     },
///     "child4" => {
///         ("token2", 1)
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

impl<'a, U, T: 'a, I: 'a, P: 'a> Iterator for SkipTokens<U>
where
    U: Iterator<Item = Node<'a, T, I, P>>,
{
    type Item = U::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.iter.next()?;

            if !matches!(node.kind(), Kind::Token) {
                return Some(node);
            }
        }
    }
}

impl<'a, U, T: 'a, I: 'a, P: 'a> DoubleEndedIterator for SkipTokens<U>
where
    U: DoubleEndedIterator<Item = Node<'a, T, I, P>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.iter.next_back()?;

            if !matches!(node.kind(), Kind::Token) {
                return Some(node);
            }
        }
    }
}

impl<'a, U, T: 'a, I: 'a, P: 'a> FusedIterator for SkipTokens<U> where
    U: FusedIterator<Item = Node<'a, T, I, P>>
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
