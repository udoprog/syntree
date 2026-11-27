//! Pointer-related types and traits.

use core::fmt;
use core::hash;

mod sealed {
    pub trait Sealed {}
}

/// A pointer type that is derived from the pointer [Width].
pub trait Pointer: Sized + Copy + hash::Hash + Eq + fmt::Debug + self::sealed::Sealed {
    /// The largest pointer value. It is reserved to mark gap entries in the
    /// token array, so no node is ever given it.
    #[doc(hidden)]
    const GAP: Self;

    /// Test if the pointer can be the id of an interior node.
    ///
    /// Interior node ids leave the top bit clear, since a token stores its
    /// trivia flag there next to its parent pointer, and the largest flagged
    /// value is [`Pointer::GAP`].
    #[doc(hidden)]
    fn is_node_id(self) -> bool;

    /// Set the flag bit on an optional node id, `None` included.
    #[doc(hidden)]
    fn flagged(this: Option<Self>) -> Option<Self>;

    /// Test if the flag bit is set on an optional node id.
    #[doc(hidden)]
    fn is_flagged(this: Option<Self>) -> bool;

    /// Clear the flag bit on an optional node id.
    #[doc(hidden)]
    fn unflagged(this: Option<Self>) -> Option<Self>;

    /// Construct a pointer without checking that `value` is in bounds.
    ///
    /// # Safety
    ///
    /// The `value` must be in bounds for the pointer type, that is
    /// [`Pointer::new(value)`][Pointer::new] must return `Some`.
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
    /// # Ok::<_, Box<dyn core::error::Error>>(())
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
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    fn get(self) -> usize;
}

/// A pointer width that can be used to reference other nodes.
///
/// This is determined by a primitive unsigned types such as `u32` or `usize`.
pub trait Width: self::sealed::Sealed {
    /// The pointer type associated with a specific width.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::pointer::Width;
    ///
    /// let tree = syntree::tree! {
    ///     "root1" => {},
    ///     "root2" => {},
    /// };
    ///
    /// let root1 = tree.first().and_then(|n| n.into_node()).ok_or("missing root")?;
    /// let root2 = tree.last().and_then(|n| n.into_node()).ok_or("missing root")?;
    ///
    /// let root1: <u32 as Width>::Pointer = root1.id();
    /// let root2: <u32 as Width>::Pointer = root2.id();
    ///
    /// assert_ne!(root1, root2);
    ///
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    type Pointer: Pointer;
}

macro_rules! implement {
    ($ty:ident, $non_zero:ident, $e:ident) => {
        impl Width for $ty {
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
            /// # Ok::<_, Box<dyn core::error::Error>>(())
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
            /// # Ok::<_, Box<dyn core::error::Error>>(())
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
        /// # Ok::<_, Box<dyn core::error::Error>>(())
        /// ```
        impl crate::pointer::Pointer for $e {
            const GAP: Self = Self($non_zero::MAX);

            #[inline]
            fn is_node_id(self) -> bool {
                // Flagging the id must neither wrap into the flag bit nor
                // produce `GAP`.
                self.0.get() < (<$ty>::MAX >> 1)
            }

            #[inline]
            fn flagged(this: Option<Self>) -> Option<Self> {
                let raw = this.map_or(0, |p| p.0.get());
                $non_zero::new(raw | !(<$ty>::MAX >> 1)).map(Self)
            }

            #[inline]
            fn is_flagged(this: Option<Self>) -> bool {
                this.is_some_and(|p| p.0.get() & !(<$ty>::MAX >> 1) != 0)
            }

            #[inline]
            fn unflagged(this: Option<Self>) -> Option<Self> {
                let raw = this.map_or(0, |p| p.0.get());
                $non_zero::new(raw & (<$ty>::MAX >> 1)).map(Self)
            }

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
                Self($non_zero::MIN)
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
