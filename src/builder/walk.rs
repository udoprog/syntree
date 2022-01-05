use std::mem;

use crate::builder::{LinkKind, Links, TreeBuilder};
use crate::non_max::NonMaxUsize;
use crate::{Kind, Span};

struct Level<'a, T> {
    /// Links for this level.
    links: &'a Links<T>,
    /// Whether or not to move down.
    down: bool,
    /// Keeps track of the generated parent node to this level.
    parent: Option<NonMaxUsize>,
    /// Keeps track of the generated previous sibling observed at this level.
    sibling: Option<NonMaxUsize>,
}

pub(crate) struct Next<'a, T> {
    pub(crate) id: NonMaxUsize,
    pub(crate) links: &'a Links<T>,
    pub(crate) kind: Kind,
    pub(crate) span: Span,
    pub(crate) parent: Option<NonMaxUsize>,
    pub(crate) first: Option<NonMaxUsize>,
    pub(crate) sibling: Option<NonMaxUsize>,
}

pub(crate) struct Up {
    pub(crate) parent: Option<NonMaxUsize>,
    pub(crate) cursor: usize,
}

pub(crate) enum Dir<'a, T> {
    Next(Next<'a, T>),
    Up(Up),
}

pub(crate) struct Walk<'a, T> {
    stack: Vec<Level<'a, T>>,
    id: usize,
    tree: &'a [Links<T>],
    cursor: usize,
}

impl<'a, T> Walk<'a, T> {
    pub(crate) fn new(builder: &'a TreeBuilder<T>) -> Self {
        let mut stack = Vec::with_capacity(if builder.data.is_empty() { 0 } else { 1 });

        if let Some(links) = builder.get(0) {
            stack.push(Level {
                links,
                down: true,
                parent: None,
                sibling: None,
            });
        }

        Self {
            stack,
            id: 0,
            tree: &builder.data,
            cursor: 0,
        }
    }
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = Dir<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let step = self.stack.last_mut()?;

            if !mem::take(&mut step.down) {
                // Move to the next element.
                if let Some(links) = step.links.next.and_then(|id| self.tree.get(id.get())) {
                    *step = Level {
                        links,
                        down: true,
                        parent: step.parent,
                        sibling: step.sibling,
                    };

                    continue;
                }

                let step = self.stack.pop()?;

                return Some(Dir::Up(Up {
                    parent: step.parent,
                    cursor: self.cursor,
                }));
            }

            let id = NonMaxUsize::new(self.id).expect("ran out of ids");
            self.id = self.id.checked_add(1).expect("ran out of ids");

            let first = if step.links.first.is_some() {
                // We can predict that the first child is `id + 1`.
                let id = id.get().checked_add(1).and_then(NonMaxUsize::new);
                Some(id.expect("ran out of ids"))
            } else {
                None
            };

            let (kind, span) = match step.links.kind {
                LinkKind::Node => (Kind::Node, Span::point(self.cursor)),
                LinkKind::Token(len) => {
                    let end = self.cursor.checked_add(len).expect("cursor out of bounds");
                    (Kind::Token, Span::new(self.cursor, end))
                }
            };

            self.cursor = span.end;

            let item = Next {
                id,
                links: step.links,
                kind,
                span,
                parent: step.parent,
                first,
                sibling: mem::replace(&mut step.sibling, Some(id)),
            };

            if let Some(links) = step.links.first.and_then(|id| self.tree.get(id.get())) {
                self.stack.push(Level {
                    links,
                    down: true,
                    parent: Some(id),
                    sibling: None,
                });
            }

            return Some(Dir::Next(item));
        }
    }
}
