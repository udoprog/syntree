use crate::index::{Index, Indexes, Length};

/// The empty [Index] implementation.
///
/// This can be used in combination with [`Builder::new_with`] to ensure that
/// that tree does not store any information on spans, reducing overhead if this
/// is not necessary.
///
/// [`Builder::new_with`]: crate::Builder::new_with
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Empty;

impl From<u32> for Empty {
    fn from(_: u32) -> Self {
        Empty
    }
}

impl From<usize> for Empty {
    fn from(_: usize) -> Self {
        Empty
    }
}

impl Index for Empty {
    const EMPTY: Self = Empty;

    type Indexes<P> = Empty where P: Copy;
    type Length = Empty;

    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn as_usize(self) -> usize {
        0
    }

    #[inline]
    fn checked_add_len(self, _: Self::Length) -> Option<Self> {
        Some(Empty)
    }

    #[inline]
    fn len_to(self, _: Self) -> Self {
        Empty
    }

    #[inline]
    fn from_usize(_: usize) -> Option<Self> {
        Some(Empty)
    }
}

impl From<Empty> for usize {
    #[inline]
    fn from(Empty: Empty) -> Self {
        0
    }
}

impl<I, P> Indexes<I, P> for Empty {
    const EMPTY: Self = Self;

    #[inline]
    fn push(&mut self, _: I, _: P) {}

    #[inline]
    fn binary_search(&self, _: I) -> Result<usize, usize> {
        Err(0)
    }

    #[inline]
    fn get(&self, _: usize) -> Option<P> {
        None
    }
}

impl Length for Empty {
    const EMPTY: Self = Empty;

    #[inline]
    fn is_empty(&self) -> bool {
        true
    }
}
