use std::mem;

use crate::builder::{Links, TreeBuilder};
use crate::non_max::NonMaxUsize;

struct Level<'a, T> {
    links: &'a Links<T>,
    /// Keeps track of the parent to this level.
    parent: Option<NonMaxUsize>,
    /// Keeps track of the last sibling observed at the given level.
    sibling: Option<NonMaxUsize>,
}

pub(crate) struct Step<'a, T> {
    pub(crate) id: NonMaxUsize,
    pub(crate) links: &'a Links<T>,
    pub(crate) parent: Option<NonMaxUsize>,
    pub(crate) first: Option<NonMaxUsize>,
    pub(crate) sibling: Option<NonMaxUsize>,
}

pub(crate) struct Walk<'a, T> {
    stack: Vec<Level<'a, T>>,
    id: usize,
    tree: &'a [Links<T>],
}

impl<'a, T> Walk<'a, T> {
    pub(crate) fn new(builder: &'a TreeBuilder<T>) -> Self {
        let mut stack = Vec::with_capacity(if builder.data.is_empty() { 0 } else { 1 });

        if let Some(links) = builder.get(0) {
            stack.push(Level {
                links,
                parent: None,
                sibling: None,
            });
        }

        Self {
            stack,
            id: 0,
            tree: &builder.data,
        }
    }

    /// Walk the tree upwards again to find the next existing sibling.
    fn next_step(&mut self, id: NonMaxUsize, links: &'a Links<T>) -> Option<Level<'a, T>> {
        if let Some(links) = links.first.and_then(|id| self.tree.get(id.get())) {
            return Some(Level {
                links,
                parent: Some(id),
                sibling: None,
            });
        }

        loop {
            let h = self.stack.pop()?;

            if let Some(links) = h.links.next.and_then(|id| self.tree.get(id.get())) {
                return Some(Level {
                    links,
                    parent: h.parent,
                    sibling: h.sibling,
                });
            }
        }
    }
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Step<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let step = self.stack.last_mut()?;

        let id = NonMaxUsize::new(self.id).expect("ran out of ids");
        self.id = self.id.checked_add(1).expect("ran out of ids");

        let first = if step.links.first.is_some() {
            // We can predict that the first child is `id + 1`.
            let id = id.get().checked_add(1).and_then(NonMaxUsize::new);
            Some(id.expect("ran out of ids"))
        } else {
            None
        };

        let item = Step {
            id,
            links: step.links,
            parent: step.parent,
            first,
            sibling: mem::replace(&mut step.sibling, Some(id)),
        };

        let links = step.links;

        if let Some(step) = self.next_step(id, links) {
            self.stack.push(step)
        }

        Some(item)
    }
}
