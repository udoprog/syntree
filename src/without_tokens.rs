use crate::{Kind, Node};

/// Wrapped around an iterator that excludes [Kind::Token] nodes.
///
/// See [Children::without_tokens][crate::Children::without_tokens] or [Walk::without_tokens][crate::Walk::without_tokens].
///
/// # Examples
///
/// Filtering [Kind::Token] elements from a [Children][crate::Children]
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
/// let mut it = tree.children().without_tokens();
///
/// assert_eq!(
///     it.map(|n| *n.data()).collect::<Vec<_>>(),
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
/// let mut it = tree.walk().without_tokens();
///
/// assert_eq!(
///     it.map(|n| *n.data()).collect::<Vec<_>>(),
///     ["child1", "child2", "child3", "child4"]
/// );
/// # Ok(()) }
/// ```
pub struct WithoutTokens<I> {
    iter: I,
}

impl<I> WithoutTokens<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'a, I, T: 'a> Iterator for WithoutTokens<I>
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

impl<'a, I, T: 'a> DoubleEndedIterator for WithoutTokens<I>
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

impl<I> Clone for WithoutTokens<I>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<I> Copy for WithoutTokens<I> where I: Copy {}
