use core::convert::Infallible;
use core::ops::DerefMut;

use crate::index::{Index, Indexes, Length};
use crate::pointer::{Pointer, Width};

/// Storage being used in a tree.
pub trait Storage<T>
where
    Self: Sized,
{
    /// A storage error.
    type Error: 'static;

    /// Empty storage.
    const EMPTY: Self;

    /// Constructstorage with the given capacity.
    fn with_capacity(capacity: usize) -> Result<Self, Self::Error>;

    /// Get the capacity of the underlying storage.
    fn capacity(&self) -> usize;

    /// Push an item into storage.
    fn push(&mut self, item: T) -> Result<(), Self::Error>;
}

impl<T> Storage<T> for alloc::vec::Vec<T> {
    type Error = Infallible;

    const EMPTY: Self = alloc::vec::Vec::new();

    #[inline]
    fn with_capacity(capacity: usize) -> Result<Self, Self::Error> {
        Ok(alloc::vec::Vec::with_capacity(capacity))
    }

    #[inline]
    fn capacity(&self) -> usize {
        alloc::vec::Vec::capacity(self)
    }

    #[inline]
    fn push(&mut self, item: T) -> Result<(), Self::Error> {
        alloc::vec::Vec::push(self, item);
        Ok(())
    }
}

/// Declare a new flavor.
///
/// The available type parameters are:
/// * `type Index` which declares the index to use.
/// * `type Width` which declares the width to use, defaults to `usize`.
///
/// # Examples
///
/// ```
/// use syntree::Empty;
///
/// syntree::flavor! {
///     struct FlavorEmpty {
///         type Index = Empty;
///         type Indexes = Empty;
///     }
/// }
///
/// syntree::flavor! {
///     struct FlavorU32 {
///         type Index = u32;
///         type Width = u32;
///     }
/// }
/// ```
#[macro_export]
macro_rules! flavor {
    (
        $(#[doc = $doc:literal])*
        $vis:vis struct $ty:ident {
            type Index = $index:ty;
            $(type Width = $width:ty;)?
            $(type Storage = $storage:ty;)?
            $(type Indexes = $indexes:ty;)?
        }
    ) => {
        $(#[doc = $doc])*
        #[non_exhaustive]
        $vis struct $ty;

        impl $crate::Flavor for $ty {
            type Error = core::convert::Infallible;
            type Index = $index;
            type Length = <$index as $crate::index::Index>::Length;
            type Width = $crate::flavor!(@width $($width)*);
            type Pointer = $crate::flavor!(@pointer $($width)*);
            type Storage<T> = $crate::macro_support::Vec<T>;
            type Indexes = $crate::flavor!(@indexes $index, $($indexes)*);
        }
    };

    (@width $ty:ty) => { $ty };
    (@width) => { usize };
    (@pointer $ty:ty) => { <$ty as $crate::pointer::Width>::Pointer };
    (@pointer) => { <usize as $crate::pointer::Width>::Pointer };
    (@indexes $index:ty, $ty:ty) => { $ty };
    (@indexes $index:ty,) => { $crate::macro_support::Vec<$crate::macro_support::TreeIndex<$index, Self::Pointer>> };
}

flavor! {
    /// The default flavor of a tree.
    ///
    /// This corresponds to a `u32` index with a `usize` width.
    pub struct FlavorDefault {
        type Index = u32;
        type Width = usize;
    }
}

/// The flavor of a tree.
///
/// This should not be implemented directly, instead see the [flavor!] macro.
///
/// The Index associated type is constrained by the [Index] trait, and
/// determines the numerical bounds of [spans] the tree.
///
/// The `Width` associated type determines the bounds of pointers in the tree
/// through the [Width] trait, this decides how many elements that can be stored
/// in the tree.
///
/// [spans]: crate::Span
pub trait Flavor {
    /// The error raised by the type of the tree.
    type Error;
    /// The type of an index used by a tree.
    type Index: Index<Length = Self::Length>;
    /// The length used in the flavor.
    #[doc(hidden)]
    type Length: Length;
    /// The width used in the flavor.
    type Width: Width<Pointer = Self::Pointer>;
    /// The pointer in use.
    #[doc(hidden)]
    type Pointer: Pointer;
    /// The storage type used in the tree.
    type Storage<T>: Storage<T, Error = Self::Error> + DerefMut<Target = [T]>;
    /// How indexes are stored in the tree.
    #[doc(hidden)]
    type Indexes: Indexes<Self::Index, Self::Pointer, Error = Self::Error>;
}
