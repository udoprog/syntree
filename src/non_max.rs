use std::fmt;

/// Helper struct which behaves exactly like `NonZeroUsize` except that it
/// rejects max values.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct NonMaxUsize(core::num::NonZeroUsize);

impl NonMaxUsize {
    #[inline]
    pub const fn new(value: usize) -> Option<Self> {
        match core::num::NonZeroUsize::new(value ^ usize::MAX) {
            None => None,
            Some(value) => Some(Self(value)),
        }
    }

    #[inline]
    pub const fn get(&self) -> usize {
        self.0.get() ^ usize::MAX
    }
}

impl fmt::Debug for NonMaxUsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}
