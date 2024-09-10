use core::convert::Infallible;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::flavor::Storage;
use crate::index::{Index, Length};

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

impl Length for Empty {
    const EMPTY: Self = Empty;

    #[inline]
    fn is_empty(&self) -> bool {
        true
    }
}

/// An empty vector.
pub struct EmptyVec<T>(PhantomData<T>);

impl<T> Default for EmptyVec<T> {
    fn default() -> Self {
        EmptyVec(PhantomData)
    }
}

impl<T> Storage<T> for EmptyVec<T> {
    const EMPTY: Self = EmptyVec(PhantomData);

    type Error = Infallible;

    #[inline]
    fn push(&mut self, _: T) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn with_capacity(_: usize) -> Result<Self, Self::Error> {
        Ok(Self::EMPTY)
    }

    #[inline]
    fn capacity(&self) -> usize {
        0
    }
}

impl<T> Deref for EmptyVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &[]
    }
}

impl<T> DerefMut for EmptyVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut []
    }
}
