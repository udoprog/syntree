use std::iter::FusedIterator;

use crate::{Kind, Node};

/// Wrapped around an iterator that excludes [Kind::Token] nodes.
///
/// See [Nodes::skip_tokens][crate::Nodes::skip_tokens] or [Walk::skip_tokens][crate::Walk::skip_tokens].
///
/// # Examples
///
/// Filtering [Kind::Token] elements from a [Nodes][crate::Nodes]
/// iterator:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tree = syntree::tree! {
///     ("token1", 1),
///     "child1",
///     ("token2", 1),
///     "child2",
///     ("token3", 1),
///     "child3",
///     ("token4", 1)
/// };
///
/// let mut it = tree.children().skip_tokens();
///
/// assert_eq!(
///     it.map(|n| *n.value()).collect::<Vec<_>>(),
///     ["child1", "child2", "child3"]
/// );
/// # Ok(()) }
/// ```
///
/// Filtering [Kind::Token] elements from a [Walk][crate::Walk] iterator:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tree = syntree::tree! {
///     "child1" => {
///         "child2",
///         ("token1", 1),
///         "child3",
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
/// # Ok(()) }
/// ```
pub struct SkipTokens<I> {
    iter: I,
}

impl<I> SkipTokens<I> {
    pub(crate) const fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'a, I, T: 'a> Iterator for SkipTokens<I>
where
    I: Iterator<Item = Node<'a, T>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.iter.next()?;

            if !matches!(node.kind(), Kind::Token) {
                return Some(node);
            }
        }
    }
}

impl<'a, I, T: 'a> DoubleEndedIterator for SkipTokens<I>
where
    I: DoubleEndedIterator<Item = Node<'a, T>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.iter.next_back()?;

            if !matches!(node.kind(), Kind::Token) {
                return Some(node);
            }
        }
    }
}

impl<'a, I, T: 'a> FusedIterator for SkipTokens<I> where I: FusedIterator<Item = Node<'a, T>> {}

impl<I> Clone for SkipTokens<I>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<I> Copy for SkipTokens<I> where I: Copy {}

impl<I> Default for SkipTokens<I>
where
    I: Default,
{
    fn default() -> Self {
        Self {
            iter: Default::default(),
        }
    }
}
