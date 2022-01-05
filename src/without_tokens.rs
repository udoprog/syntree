use crate::{Kind, Node};

/// Wrapped around an iterator that excludes [Kind::Token] nodes.
///
/// See [Children::without_tokens][crate::Children::without_tokens].
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
