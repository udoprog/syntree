//! Trait for defining the flavor of a syntree.

use core::ops::DerefMut;

use crate::index::{Index, Length, TreeIndex};
use crate::pointer::{Pointer, Width};

/// Storage being used in a tree.
pub trait Storage<T>
where
    Self: Sized + DerefMut<Target = [T]>,
{
    /// A storage error.
    type Error: 'static;

    /// Empty storage.
    const EMPTY: Self;

    /// Construct storage with the given capacity.
    fn with_capacity(capacity: usize) -> Result<Self, Self::Error>;

    /// Get the capacity of the underlying storage.
    fn capacity(&self) -> usize;

    /// Push an item into storage.
    fn push(&mut self, item: T) -> Result<(), Self::Error>;
}

#[cfg(feature = "alloc")]
impl<T> Storage<T> for alloc::vec::Vec<T> {
    type Error = core::convert::Infallible;

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
/// The associated types are declared in this order, and all but `Index` are
/// optional:
/// * `type Index = ..;` the [Index] used for spans.
/// * `type Width = ..;` the [Width] which bounds pointers, defaults to `usize`.
/// * `type Error = ..;` the error raised by storage, defaults to
///   [`Infallible`].
/// * `type Storage<T> = ..;` the [Storage] used for tree data, defaults to
///   `Vec<T>` (or [`EmptyVec<T>`] without the `alloc` feature).
/// * `type Indexes = ..;` the [Storage] of [`TreeIndex<Self>`], defaults to
///   `Vec<TreeIndex<Self>>` (or [`EmptyVec`] without the `alloc` feature),
///   regardless of `Storage`.
///
/// The `Error` of both `Storage` and `Indexes` must be the flavor's `Error`,
/// so a flavor with a custom `Error` needs to declare both of them. Errors
/// raised by storage are reported as [`Error::Flavor`].
///
/// [`Infallible`]: core::convert::Infallible
/// [`EmptyVec<T>`]: crate::EmptyVec
/// [`EmptyVec`]: crate::EmptyVec
/// [`Error::Flavor`]: crate::Error::Flavor
///
/// # Examples
///
/// ```
/// use syntree::{Empty, EmptyVec, TreeIndex};
///
/// syntree::flavor! {
///     struct FlavorEmpty {
///         type Index = Empty;
///         type Indexes = EmptyVec<TreeIndex<Self>>;
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
///
/// A flavor with custom fallible storage:
///
/// ```
/// use core::ops::{Deref, DerefMut};
/// use syntree::{Builder, Error, Storage, TreeIndex};
///
/// #[derive(Debug, PartialEq)]
/// struct Full;
///
/// /// Storage which holds at most one item.
/// struct One<T>(Option<T>);
///
/// impl<T> Deref for One<T> {
///     type Target = [T];
///
///     fn deref(&self) -> &[T] {
///         self.0.as_slice()
///     }
/// }
///
/// impl<T> DerefMut for One<T> {
///     fn deref_mut(&mut self) -> &mut [T] {
///         self.0.as_mut_slice()
///     }
/// }
///
/// impl<T> Storage<T> for One<T> {
///     type Error = Full;
///
///     const EMPTY: Self = One(None);
///
///     fn with_capacity(capacity: usize) -> Result<Self, Full> {
///         if capacity > 1 {
///             return Err(Full);
///         }
///
///         Ok(One(None))
///     }
///
///     fn capacity(&self) -> usize {
///         1
///     }
///
///     fn push(&mut self, item: T) -> Result<(), Full> {
///         if self.0.is_some() {
///             return Err(Full);
///         }
///
///         self.0 = Some(item);
///         Ok(())
///     }
/// }
///
/// syntree::flavor! {
///     struct FlavorOne {
///         type Index = u32;
///         type Error = Full;
///         type Storage<T> = One<T>;
///         type Indexes = One<TreeIndex<Self>>;
///     }
/// }
///
/// let mut tree: Builder<_, FlavorOne> = Builder::new_with();
/// tree.token("a", 1)?;
/// assert_eq!(tree.token("b", 1), Err(Error::Flavor(Full)));
/// # Ok::<_, Error<Full>>(())
/// ```
#[macro_export]
macro_rules! flavor {
    (
        $(#[doc = $doc:literal])*
        $vis:vis struct $ty:ident {
            type Index = $index:ty;
            $(type Width = $width:ty;)?
            $(type Error = $error:ty;)?
            $(type Storage<$t:ident> = $storage:ty;)?
            $(type Indexes = $indexes:ty;)?
        }
    ) => {
        $(#[doc = $doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        $vis struct $ty;

        impl $crate::Flavor for $ty {
            type Error = $crate::flavor!(@error $($error)*);
            type Index = $index;
            type Length = <$index as $crate::Index>::Length;
            type Width = $crate::flavor!(@width $($width)*);
            type Pointer = $crate::flavor!(@pointer $($width)*);
            $crate::flavor!(@storage $($t $storage)*);
            type Indexes = $crate::flavor!(@indexes $($indexes)*);
        }
    };

    (@error $ty:ty) => { $ty };
    (@error) => { ::core::convert::Infallible };
    (@width $ty:ty) => { $ty };
    (@width) => { usize };
    (@pointer $ty:ty) => { <$ty as $crate::pointer::Width>::Pointer };
    (@pointer) => { <usize as $crate::pointer::Width>::Pointer };
    (@storage $t:ident $ty:ty) => { type Storage<$t> = $ty; };
    (@storage) => { type Storage<T> = $crate::macro_support::Vec<T>; };
    (@indexes $ty:ty) => { $ty };
    (@indexes) => { $crate::macro_support::DefaultIndexes<Self> };
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
    type Length: Length;
    /// The width used in the flavor.
    type Width: Width<Pointer = Self::Pointer>;
    /// The pointer in use.
    type Pointer: Pointer;
    /// The storage type used in the tree.
    type Storage<T>: Storage<T, Error = Self::Error>;
    /// How indexes are stored in the tree.
    type Indexes: Storage<TreeIndex<Self>, Error = Self::Error>;
}
