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

    /// Safe constructor for the pointer.
    ///
    /// Returns [`None`] if the value provided is out of bounds for the pointer type.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::pointer::{Pointer, PointerU32};
    ///
    /// let v = PointerU32::new(0).ok_or("out of bounds")?;
    /// assert_eq!(v.get(), 0);
    /// let v = PointerU32::new(42).ok_or("out of bounds")?;
    /// assert_eq!(v.get(), 42);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn new(value: usize) -> Option<Self>;

    /// Get the index of a pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::pointer::{Pointer, PointerU32};
    ///
    /// let v = PointerU32::new(0).ok_or("out of bounds")?;
    /// assert_eq!(v.get(), 0);
    /// let v = PointerU32::new(42).ok_or("out of bounds")?;
    /// assert_eq!(v.get(), 42);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
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
    ($ty:ident, $non_zero:ident, $e:ident) => {
        impl Width for $ty {
            const EMPTY: Self = 0;
            type Pointer = $e;
        }

        use core::num::$non_zero;

        impl self::sealed::Sealed for $ty {}
        impl self::sealed::Sealed for $e {}

        #[doc = concat!(" [`Pointer`] implementation for `", stringify!($ty), "`.")]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $e($non_zero);

        impl $e {
            #[doc = concat!(" Safe constructor for the pointer.")]
            ///
            #[doc = concat!(" Returns [`None`] if the value provided is out of bounds for the pointer type.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use syntree::pointer::Pointer;
            #[doc = concat!("use syntree::pointer::", stringify!($e), ";")]
            ///
            #[doc = concat!(" let v = ", stringify!($e), "::new(0).ok_or(\"out of bounds\")?;")]
            /// assert_eq!(v.get(), 0);
            #[doc = concat!(" let v = ", stringify!($e), "::new(42).ok_or(\"out of bounds\")?;")]
            /// assert_eq!(v.get(), 42);
            /// # Ok::<_, Box<dyn std::error::Error>>(())
            /// ```
            #[inline]
            pub fn new(value: usize) -> Option<Self> {
                let value = <$ty>::try_from(value).ok()?;
                $non_zero::new(value.wrapping_add(1)).map(Self)
            }

            #[doc = concat!(" Get the index of a pointer.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use syntree::pointer::Pointer;
            #[doc = concat!("use syntree::pointer::", stringify!($e), ";")]
            ///
            #[doc = concat!(" let v = ", stringify!($e), "::new(0).ok_or(\"out of bounds\")?;")]
            /// assert_eq!(v.get(), 0);
            #[doc = concat!(" let v = ", stringify!($e), "::new(42).ok_or(\"out of bounds\")?;")]
            /// assert_eq!(v.get(), 42);
            /// # Ok::<_, Box<dyn std::error::Error>>(())
            /// ```
            #[inline]
            pub fn get(self) -> usize {
                self.0.get().wrapping_sub(1) as usize
            }
        }

        #[doc = concat!(" Pointer used for `", stringify!($ty), "` type.")]
        ///
        /// ```
        /// use syntree::pointer::Pointer;
        #[doc = concat!("use syntree::pointer::", stringify!($e), ";")]
        ///
        #[doc = concat!(" let v = ", stringify!($e), "::new(0).ok_or(\"out of bounds\")?;")]
        /// assert_eq!(v.get(), 0);
        #[doc = concat!(" let v = ", stringify!($e), "::new(42).ok_or(\"out of bounds\")?;")]
        /// assert_eq!(v.get(), 42);
        /// # Ok::<_, Box<dyn std::error::Error>>(())
        /// ```
        impl crate::pointer::Pointer for $e {
            #[inline]
            unsafe fn new_unchecked(value: usize) -> Self {
                let value = value as $ty;
                Self($non_zero::new_unchecked(value.wrapping_add(1)))
            }

            #[inline]
            fn new(value: usize) -> Option<Self> {
                $e::new(value)
            }

            #[inline]
            fn get(self) -> usize {
                $e::get(self)
            }
        }

        /// Construct a default value.
        ///
        /// ```
        /// use syntree::pointer::Pointer;
        #[doc = concat!("use syntree::pointer::", stringify!($e), ";")]
        ///
        #[doc = concat!(" let v = ", stringify!($e), "::default();")]
        /// assert_eq!(v.get(), 0);
        /// ```
        impl Default for $e {
            #[inline]
            fn default() -> Self {
                // SAFETY: we know that 1 is a legal value and that it
                // corresponds to zero.
                unsafe { Self($non_zero::new_unchecked(1)) }
            }
        }

        impl fmt::Debug for $e {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }
    };
}

implement!(usize, NonZeroUsize, PointerUsize);
implement!(u8, NonZeroU8, PointerU8);
implement!(u16, NonZeroU16, PointerU16);
implement!(u32, NonZeroU32, PointerU32);
implement!(u64, NonZeroU64, PointerU64);
implement!(u128, NonZeroU128, PointerU128);
