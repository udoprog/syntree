//! Types that can be used to refer to indexes in a [Span][crate::Span].

use core::cmp;

use alloc::vec::Vec;

mod sealed {
    use alloc::vec::Vec;

    pub trait Sealed {}

    impl Sealed for u32 {}
    impl Sealed for usize {}
    impl Sealed for crate::empty::Empty {}

    impl<I, P> Sealed for Vec<crate::index::TreeIndex<I, P>> {}
}

/// A type that can be used when referring to an index in a tree.
///
/// An index is a valid single component of a [Span][crate::Span], valid indexes
/// are types such as `u32` and `usize`, but also [`Empty`][crate::Empty] in
/// case indexing is not required.
///
/// See [Builder::new_with][crate::Builder::new_with].
pub trait Index: Sized + Copy + cmp::Ord + cmp::Eq + self::sealed::Sealed {
    #[doc(hidden)]
    const EMPTY: Self;

    #[doc(hidden)]
    type Length: Length;

    #[doc(hidden)]
    fn is_empty(&self) -> bool;

    #[doc(hidden)]
    fn as_usize(self) -> usize;

    #[doc(hidden)]
    fn checked_add_len(self, other: Self::Length) -> Option<Self>;

    #[doc(hidden)]
    fn len_to(self, other: Self) -> Self::Length;

    #[doc(hidden)]
    fn from_usize(value: usize) -> Option<Self>;
}

#[doc(hidden)]
pub trait Indexes<I, P>: self::sealed::Sealed {
    const EMPTY: Self;

    #[doc(hidden)]
    fn push(&mut self, cursor: I, id: P);

    #[doc(hidden)]
    fn get(&self, index: usize) -> Option<&P>;
}

#[doc(hidden)]
pub trait Length: Copy + self::sealed::Sealed {
    #[doc(hidden)]
    const EMPTY: Self;

    #[doc(hidden)]
    fn is_empty(&self) -> bool;
}

impl Length for usize {
    const EMPTY: Self = 0;

    #[inline]
    fn is_empty(&self) -> bool {
        *self == 0
    }
}

/// Ensure u32 is smaller or equal to usize.
const _: () = assert!(core::mem::size_of::<u32>() <= core::mem::size_of::<usize>());

impl Index for u32 {
    const EMPTY: Self = 0;

    type Length = usize;

    #[inline]
    fn is_empty(&self) -> bool {
        *self == 0
    }

    #[inline]
    fn as_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn checked_add_len(self, other: Self::Length) -> Option<Self> {
        u32::checked_add(self, u32::try_from(other).ok()?)
    }

    #[inline]
    fn len_to(self, other: Self) -> Self::Length {
        other.saturating_sub(self) as usize
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        u32::try_from(value).ok()
    }
}

impl Index for usize {
    const EMPTY: Self = 0;

    type Length = usize;

    #[inline]
    fn is_empty(&self) -> bool {
        *self == 0
    }

    #[inline]
    fn as_usize(self) -> usize {
        self
    }

    #[inline]
    fn checked_add_len(self, other: Self::Length) -> Option<Self> {
        usize::checked_add(self, other)
    }

    #[inline]
    fn len_to(self, other: Self) -> Self::Length {
        other.saturating_sub(self)
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        Some(value)
    }
}

/// Trait which constrains [Indexes] capable of being binary searched.
///
/// This allows search operations by span to be performed over a [Tree].
///
/// [Tree]: crate::Tree
pub trait BinarySearch<I> {
    /// Perform a binary search for the specified index.
    fn binary_search(&self, index: I) -> Result<usize, usize>;
}

#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct TreeIndex<I, P> {
    pub(crate) index: I,
    pub(crate) id: P,
}

impl<I, P> BinarySearch<I> for [TreeIndex<I, P>]
where
    I: Ord,
{
    #[inline]
    fn binary_search(&self, index: I) -> Result<usize, usize> {
        self.binary_search_by(|f| f.index.cmp(&index))
    }
}

impl<I, P> Indexes<I, P> for Vec<TreeIndex<I, P>> {
    const EMPTY: Self = Self::new();

    #[inline]
    fn push(&mut self, index: I, id: P) {
        Vec::push(self, TreeIndex { index, id })
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&P> {
        Some(&<[_]>::get(self, index)?.id)
    }
}
