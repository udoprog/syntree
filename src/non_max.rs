/// Helper struct which behaves exactly like `NonZeroUsize` except that it
/// rejects max values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct NonMaxUsize(core::num::NonZeroUsize);

impl NonMaxUsize {
    /// Creates a new non-max if the given value is not the maximum
    /// value.
    #[inline]
    pub const fn new(value: usize) -> Option<Self> {
        match core::num::NonZeroUsize::new(value ^ usize::max_value()) {
            None => None,
            Some(value) => Some(Self(value)),
        }
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(&self) -> usize {
        self.0.get() ^ usize::max_value()
    }
}
