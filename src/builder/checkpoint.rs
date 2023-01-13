use core::cell::Cell;
use std::rc::Rc;

use crate::non_max::NonMax;

/// The identifier of a node as returned by functions such as
/// [`Builder::checkpoint`].
///
/// This can be used as a checkpoint in [`Builder::close_at`], and a checkpoint
/// can be fetched up front from [`Builder::checkpoint`].
///
/// [`Builder::close_at`]: crate::Builder::close_at
/// [`Builder::checkpoint`]: crate::Builder::checkpoint
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Checkpoint(Rc<Cell<Inner>>);

impl Checkpoint {
    pub(crate) fn new(node: NonMax, parent: Option<NonMax>) -> Self {
        Self(Rc::new(Cell::new(Inner { node, parent })))
    }

    pub(crate) fn set(&self, node: NonMax, parent: Option<NonMax>) {
        self.0.set(Inner { node, parent });
    }

    pub(crate) fn node(&self) -> NonMax {
        self.0.get().node
    }

    pub(crate) fn get(&self) -> (NonMax, Option<NonMax>) {
        let Inner { node, parent } = self.0.get();
        (node, parent)
    }
}

/// The parent of the checkpoint.
#[derive(Debug, Clone, Copy)]
struct Inner {
    // The node being wrapped by the checkpoint.
    node: NonMax,
    // The parent node of the context being checkpointed.
    parent: Option<NonMax>,
}
