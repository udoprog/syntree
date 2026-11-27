use core::fmt;
use core::marker::PhantomData;

/// A checkpoint as returned by [`Builder::checkpoint`], used to wrap
/// everything added since it was taken with [`Builder::close_at`].
///
/// A checkpoint is a `Copy` index into the builder that handed it out, and the
/// builder holds its state. Closing it moves it onto the new wrapping node, so
/// closing it again wraps the wrapper.
///
/// # Checkpoints and cloned builders
///
/// A cloned [`Builder`] clones the state of every checkpoint, so a checkpoint
/// taken before cloning can be used on both and moves independently in each.
///
/// ```
/// let mut a = syntree::Builder::new();
/// let c = a.checkpoint()?;
///
/// let mut b = a.clone();
/// b.token("lit", 1)?;
/// b.close_at(c, "speculative")?;
///
/// a.token("lit", 1)?;
/// a.close_at(c, "root")?;
///
/// assert_eq!(a.build()?, syntree::tree! { "root" => { ("lit", 1) } });
/// assert_eq!(b.build()?, syntree::tree! { "speculative" => { ("lit", 1) } });
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// [`Builder`]: crate::Builder
/// [`Builder::close_at`]: crate::Builder::close_at
/// [`Builder::checkpoint`]: crate::Builder::checkpoint
#[repr(transparent)]
pub struct Checkpoint<P> {
    index: usize,
    _marker: PhantomData<fn() -> P>,
}

impl<P> Checkpoint<P> {
    #[inline]
    pub(crate) const fn new(index: usize) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) const fn index(self) -> usize {
        self.index
    }
}

impl<P> Clone for Checkpoint<P> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for Checkpoint<P> {}

impl<P> PartialEq for Checkpoint<P> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<P> Eq for Checkpoint<P> {}

impl<P> fmt::Debug for Checkpoint<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Checkpoint").field(&self.index).finish()
    }
}

/// The state of a checkpoint, stored in the builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Inner<P> {
    /// The number of token entries when the checkpoint was taken.
    pub(crate) tok: usize,
    /// The last interior node at the level when the checkpoint was taken.
    pub(crate) before: Option<P>,
    /// The node being built when the checkpoint was taken.
    pub(crate) parent: Option<P>,
}
