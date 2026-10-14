use core::cell::Cell;

use alloc::rc::Rc;

/// The identifier of a node as returned by functions such as
/// [`Builder::checkpoint`].
///
/// This can be used as a checkpoint in [`Builder::close_at`], and a checkpoint
/// can be fetched up front from [`Builder::checkpoint`].
///
/// # Checkpoints and cloned builders
///
/// A checkpoint belongs to the builder that handed it out, and closing it
/// updates it in place. A cloned [`Builder`] hands out its own checkpoints, so
/// one taken from the clone can be used on it without affecting the original.
/// A checkpoint taken *before* cloning still belongs to the original: closing
/// it on the clone updates it for the original too, so get a fresh checkpoint
/// from the clone instead.
///
/// ```
/// let mut a = syntree::Builder::new();
/// let c = a.checkpoint()?;
///
/// let mut b = a.clone();
/// let c2 = b.checkpoint()?;
/// b.token("lit", 1)?;
/// b.close_at(&c2, "speculative")?;
///
/// a.token("lit", 1)?;
/// a.close_at(&c, "root")?;
///
/// assert_eq!(a.build()?, syntree::tree! { "root" => { ("lit", 1) } });
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// [`Builder`]: crate::Builder
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

    /// A new checkpoint with the same state which does not share updates with
    /// this one.
    pub(crate) fn deep_clone(&self) -> Self {
        Self(Rc::new(Cell::new(self.0.get())))
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

/// The shared state of a checkpoint.
#[derive(Debug, Clone, Copy)]
struct Inner<P> {
    // The node being wrapped by the checkpoint.
    node: P,
    // The parent node of the context being checkpointed.
    parent: Option<P>,
}
