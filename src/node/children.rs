use core::fmt;
use core::iter::FusedIterator;

use crate::flavor::Flavor;
use crate::node::{Element, Id};
use crate::pointer::Pointer;
use crate::tree::Tree;

/// An iterator over the children of a node or the root of a tree, nodes and
/// tokens in order.
///
/// See [`Tree::children`] or [`Node::children`][crate::Node::children].
pub struct Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    tree: &'a Tree<T, F>,
    /// The node whose children are iterated, `None` at the root.
    parent: Option<F::Pointer>,
    /// The token position and next interior node at the front.
    front: (usize, Option<F::Pointer>),
    /// The token position and previous interior node at the back.
    back: (usize, Option<F::Pointer>),
    /// The last element yielded from the front.
    front_last: Option<Id<F::Pointer>>,
    /// The last element yielded from the back.
    back_last: Option<Id<F::Pointer>>,
    done: bool,
}

impl<'a, T, F> Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    pub(crate) fn new(
        tree: &'a Tree<T, F>,
        parent: Option<F::Pointer>,
        front: (usize, Option<F::Pointer>),
        back: (usize, Option<F::Pointer>),
    ) -> Self {
        Self {
            tree,
            parent,
            front,
            back,
            front_last: None,
            back_last: None,
            done: false,
        }
    }
}

impl<'a, T, F> Iterator for Children<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    type Item = Element<'a, T, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let Some(id) = self.tree.next_at(self.parent, self.front.0, self.front.1) else {
            self.done = true;
            return None;
        };

        if Some(id) == self.back_last {
            self.done = true;
            return None;
        }

        self.front = match id {
            Id::Node(n) => {
                let links = self.tree.node_links(n)?;
                (links.tok_end.get(), links.next)
            }
            Id::Token(t) => (t.get().checked_add(1)?, self.front.1),
        };

        self.front_last = Some(id);
        self.tree.element(id)
    }
}

impl<T, F> DoubleEndedIterator for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let Some(id) = self.tree.prev_at(self.parent, self.back.0, self.back.1) else {
            self.done = true;
            return None;
        };

        if Some(id) == self.front_last {
            self.done = true;
            return None;
        }

        self.back = match id {
            Id::Node(n) => {
                let links = self.tree.node_links(n)?;
                (links.tok_start.get(), links.prev)
            }
            Id::Token(t) => (t.get(), self.back.1),
        };

        self.back_last = Some(id);
        self.tree.element(id)
    }
}

impl<T, F> FusedIterator for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, F> Clone for Children<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            parent: self.parent,
            front: self.front,
            back: self.back,
            front_last: self.front_last,
            back_last: self.back_last,
            done: self.done,
        }
    }
}

impl<T, F> fmt::Debug for Children<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
