use crate::tree::{Kind, Node};
use crate::{Children, Span, Walk};

/// Iterator over the children of a [Node] or [Tree]. This excludes [Kind::Token]
/// nodes.
///
/// See [Children::without_tokens].
pub struct WithoutTokens<I> {
    iter: I,
}

impl<I> WithoutTokens<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'a, T> WithoutTokens<Children<'a, T>> {
    /// Calculate the span of the remaining nodes in the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Span};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     "number" => {
    ///         ("number", 5)
    ///     },
    ///     "ident" => {
    ///         ("ident", 2)
    ///     }
    /// };
    ///
    /// let mut it = tree.children().without_tokens();
    ///
    /// it.next();
    ///
    /// assert_eq!(it.span(), Some(Span::new(5, 7)));
    /// # Ok(()) }
    /// ```
    pub fn span(self) -> Option<Span> {
        self.iter.span()
    }

    /// Walk the rest of the tree forwards in a depth-first fashion.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "c1" => {
    ///             ("c2", 5),
    ///             "c3",
    ///             "c4",
    ///         },
    ///         ("c5", 5),
    ///         "c6"
    ///     }
    /// };
    ///
    /// let it = tree.children().without_tokens();
    ///
    /// let nodes = it.walk().rev().map(|n| *n.data()).collect::<Vec<_>>();
    /// assert_eq!(nodes, vec!["root", "c6", "c1", "c4", "c3"]);
    /// # Ok(()) }
    /// ```
    pub fn walk(self) -> WithoutTokens<Walk<'a, T>> {
        WithoutTokens::new(self.iter.walk())
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

            if !matches!(node.kind(), Kind::Token(..)) {
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

            if !matches!(node.kind(), Kind::Token(..)) {
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
