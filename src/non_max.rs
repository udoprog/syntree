/// Get the configured nonmax type.
#[cfg(not(feature = "u32"))]
pub(crate) type NonMax = self::imp::NonMaxUsize;
#[cfg(feature = "u32")]
pub(crate) type NonMax = self::imp::NonMaxU32;

#[cfg(not(feature = "u32"))]
mod imp {
    use std::fmt;
    use std::num::NonZeroUsize;

    /// Helper struct which behaves exactly like `NonZeroUsize` except that it
    /// rejects max values.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub(crate) struct NonMaxUsize(NonZeroUsize);

    impl NonMaxUsize {
        /// Unchecked constructor.
        #[inline]
        pub(crate) unsafe fn new_unchecked(value: usize) -> Self {
            Self(NonZeroUsize::new_unchecked(value ^ usize::MAX))
        }

        #[inline]
        pub(crate) const fn new(value: usize) -> Option<Self> {
            match NonZeroUsize::new(value ^ usize::MAX) {
                None => None,
                Some(value) => Some(Self(value)),
            }
        }

        #[inline]
        pub(crate) const fn get(&self) -> usize {
            self.0.get() ^ usize::MAX
        }
    }

    impl fmt::Debug for NonMaxUsize {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.get().fmt(f)
        }
    }
}

#[cfg(feature = "u32")]
mod imp {
    use std::fmt;
    use std::mem::size_of;
    use std::num::NonZeroU32;

    /// Helper struct which behaves exactly like `NonZeroU32` except that it rejects
    /// max values.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub(crate) struct NonMaxU32(NonZeroU32);

    impl NonMaxU32 {
        /// Unchecked constructor.
        #[inline]
        pub(crate) unsafe fn new_unchecked(value: usize) -> Self {
            let value = value as u32;
            Self(NonZeroU32::new_unchecked(value ^ u32::MAX))
        }

        #[inline]
        pub(crate) const fn new(value: usize) -> Option<Self> {
            let value = if size_of::<usize>() <= size_of::<u32>() {
                value as u32
            } else {
                if value > u32::MAX as usize {
                    return None;
                }

                value as u32
            };

            match NonZeroU32::new((value as u32) ^ u32::MAX) {
                None => None,
                Some(value) => Some(Self(value)),
            }
        }

        #[inline]
        pub(crate) const fn get(&self) -> usize {
            (self.0.get() ^ u32::MAX) as usize
        }
    }

    impl fmt::Debug for NonMaxU32 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.get().fmt(f)
        }
    }
}
