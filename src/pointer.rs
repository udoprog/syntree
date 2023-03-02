//! Pointer-related types and traits.

use core::fmt;
use core::hash;

mod sealed {
    pub trait Sealed {}
}

/// A pointer type that is derived from the pointer [Width].
#[doc(hidden)]
pub trait Pointer: Sized + Copy + hash::Hash + Eq + fmt::Debug + self::sealed::Sealed {
    #[doc(hidden)]
    unsafe fn new_unchecked(value: usize) -> Self;
    #[doc(hidden)]
    fn new(value: usize) -> Option<Self>;
    #[doc(hidden)]
    fn get(self) -> usize;
}

/// A pointer width that can be used to reference other nodes.
///
/// This is determined by a primitive unsigned types such as `u32` or `usize`.
pub trait Width: self::sealed::Sealed {
    #[doc(hidden)]
    const EMPTY: Self;

    /// The pointer type associated with a specific width.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::pointer::Width;
    ///
    /// let tree = syntree::tree! {
    ///     "root1",
    ///     "root2",
    /// };
    ///
    /// let root1: <usize as Width>::Pointer = tree.first().ok_or("missing root")?.id();
    /// let root2: <usize as Width>::Pointer = tree.last().ok_or("missing root")?.id();
    ///
    /// assert_ne!(root1, root2);
    ///
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    type Pointer: Pointer;
}

macro_rules! implement {
    ($ty:ident, $non_zero:ident) => {
        impl Width for $ty {
            const EMPTY: Self = 0;
            type Pointer = self::$ty::Pointer;
        }

        impl self::sealed::Sealed for $ty {}
        impl self::sealed::Sealed for self::$ty::Pointer {}

        mod $ty {
            use core::fmt;
            use core::mem::size_of;
            use core::num::$non_zero;

            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct Pointer($non_zero);

            impl crate::pointer::Pointer for Pointer {
                #[inline]
                unsafe fn new_unchecked(value: usize) -> Self {
                    let value = value as $ty;
                    Self($non_zero::new_unchecked(value ^ <$ty>::MAX))
                }

                #[inline]
                fn new(value: usize) -> Option<Self> {
                    let value = if size_of::<usize>() <= size_of::<$ty>() {
                        value as $ty
                    } else {
                        if value > <$ty>::MAX as usize {
                            return None;
                        }

                        value as $ty
                    };

                    match $non_zero::new((value as $ty) ^ <$ty>::MAX) {
                        None => None,
                        Some(value) => Some(Self(value)),
                    }
                }

                #[inline]
                fn get(self) -> usize {
                    (self.0.get() ^ <$ty>::MAX) as usize
                }
            }

            impl Default for Pointer {
                #[inline]
                fn default() -> Self {
                    // SAFETY: we know that max is a legal value and that it
                    // corresponds to zero.
                    unsafe { Self($non_zero::new_unchecked(<$ty>::MAX)) }
                }
            }

            impl fmt::Debug for Pointer {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    (self.0.get() ^ <$ty>::MAX).fmt(f)
                }
            }
        }
    };
}

implement!(usize, NonZeroUsize);
implement!(u8, NonZeroU8);
implement!(u16, NonZeroU16);
implement!(u32, NonZeroU32);
implement!(u64, NonZeroU64);
implement!(u128, NonZeroU128);