//! Pointer-related types and traits.

use core::fmt;
use core::hash;

mod sealed {
    pub trait Sealed {}
}

/// A pointer type that is derived from the pointer [Width].
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
/// This is determined by a primitive unsigned type such as `u32` or `usize.
pub trait Width: self::sealed::Sealed {
    #[doc(hidden)]
    type Pointer: Pointer;
}

macro_rules! implement {
    ($name:ident, $ty:ty, $base:ident) => {
        impl Width for $ty {
            type Pointer = self::$name::NonMax;
        }

        impl self::sealed::Sealed for $ty {}
        impl self::sealed::Sealed for self::$name::NonMax {}

        mod $name {
            use core::fmt;
            use core::mem::size_of;
            use core::num::$base;

            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct NonMax($base);

            impl crate::pointer::Pointer for NonMax {
                #[inline]
                unsafe fn new_unchecked(value: usize) -> Self {
                    let value = value as $ty;
                    Self($base::new_unchecked(value ^ <$ty>::MAX))
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

                    match $base::new((value as $ty) ^ <$ty>::MAX) {
                        None => None,
                        Some(value) => Some(Self(value)),
                    }
                }

                #[inline]
                fn get(self) -> usize {
                    (self.0.get() ^ <$ty>::MAX) as usize
                }
            }

            impl fmt::Debug for NonMax {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    (self.0.get() ^ <$ty>::MAX).fmt(f)
                }
            }
        }
    };
}

implement!(usize, usize, NonZeroUsize);
implement!(u16, u16, NonZeroU16);
implement!(u32, u32, NonZeroU32);
implement!(u64, u64, NonZeroU64);
