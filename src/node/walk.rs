use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::node::{Element, Event, Id, WalkEvents};
use crate::tree::Tree;

/// An iterator that walks forward over the tree in a depth-first fashion,
/// visiting every node and token once.
///
/// See [`Tree::walk`][crate::Tree::walk] or [`Node::walk`][crate::Node::walk].
pub struct Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    iter: WalkEvents<'a, T, F>,
}

impl<'a, T, F> Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Walk from `start`, where `next` is the next interior node at the level
    /// of `start` if it is a token.
    #[inline]
    pub(crate) fn new(
        tree: &'a Tree<T, F>,
        start: Option<Id<F::Pointer>>,
        next: Option<F::Pointer>,
    ) -> Self {
        Self {
            iter: WalkEvents::new(tree, start, next),
        }
    }

    /// Convert this into an iterator which yields each element with its
    /// depth relative to where the walk started.
    #[inline]
    #[must_use]
    pub fn with_depths(self) -> WithDepths<'a, T, F> {
        WithDepths { iter: self.iter }
    }

    /// Get the next element with its depth.
    #[inline]
    #[must_use]
    pub fn next_with_depth(&mut self) -> Option<(isize, Element<'a, T, F>)> {
        next_with_depth(&mut self.iter)
    }
}

impl<T, F> Clone for Walk<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, F> Default for Walk<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
        }
    }
}

impl<'a, T, F> Iterator for Walk<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Element<'a, T, F>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (event, element) = self.iter.next()?;

            if !matches!(event, Event::Up) {
                return Some(element);
            }
        }
    }
}

impl<T, F> FusedIterator for Walk<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

/// An iterator which walks the tree and yields each element with its depth.
///
/// See [`Walk::with_depths`].
pub struct WithDepths<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    iter: WalkEvents<'a, T, F>,
}

impl<'a, T, F> Iterator for WithDepths<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = (isize, Element<'a, T, F>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        next_with_depth(&mut self.iter)
    }
}

#[inline]
fn next_with_depth<'a, T, F>(iter: &mut WalkEvents<'a, T, F>) -> Option<(isize, Element<'a, T, F>)>
where
    T: Copy,
    F: Flavor,
{
    loop {
        let depth = iter.depth();
        let (event, element) = iter.next()?;

        if !matches!(event, Event::Up) {
            return Some((depth, element));
        }
    }
}

impl<T, F> FusedIterator for WithDepths<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for WithDepths<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<T, F> Default for WithDepths<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            iter: WalkEvents::default(),
        }
    }
}

impl<T, F> fmt::Debug for Walk<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Walk").field("events", &self.iter).finish()
    }
}

impl<T, F> fmt::Debug for WithDepths<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WithDepths")
            .field("events", &self.iter)
            .finish()
    }
}
