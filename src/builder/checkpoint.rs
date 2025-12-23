use core::cell::Cell;

use alloc::rc::Rc;

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
pub struct Checkpoint<P>(Rc<Cell<Inner<P>>>)
where
    P: Copy;

impl<P> Checkpoint<P>
where
    P: Copy,
{
    pub(crate) fn new(node: P, parent: Option<P>) -> Self {
        Self(Rc::new(Cell::new(Inner { node, parent })))
    }

    pub(crate) fn set(&self, node: P, parent: Option<P>) {
        self.0.set(Inner { node, parent });
    }

    pub(crate) fn node(&self) -> P {
        self.0.get().node
    }

    pub(crate) fn parent(&self) -> Option<P> {
        self.0.get().parent
    }

    pub(crate) fn get(&self) -> (P, Option<P>) {
        let Inner { node, parent } = self.0.get();
        (node, parent)
    }
}

/// The parent of the checkpoint.
#[derive(Debug, Clone, Copy)]
struct Inner<P> {
    // The node being wrapped by the checkpoint.
    node: P,
    // The parent node of the context being checkpointed.
    parent: Option<P>,
}
