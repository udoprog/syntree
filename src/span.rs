//! Types to deal with spans in syntax trees.

use core::cmp;
use core::fmt;
use core::ops;

mod sealed {
    pub trait Sealed {}

    impl Sealed for u32 {}
    impl Sealed for usize {}
    impl Sealed for super::Empty {}

    impl<I, P> Sealed for Vec<super::TreeIndex<I, P>> where I: super::Index {}
}

/// Ensure u32 is smaller or equal to usize.
const _: () = assert!(core::mem::size_of::<u32>() <= core::mem::size_of::<usize>());

/// A type that can be used as an index in a tree.
pub trait Index: Sized + Copy + cmp::Ord + cmp::Eq + self::sealed::Sealed {
    #[doc(hidden)]
    const EMPTY: Self;
    #[doc(hidden)]
    type Indexes<P>: Indexes<Self, P>
    where
        P: Copy;
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
    fn saturating_sub(self, other: Self) -> Self;
    #[doc(hidden)]
    fn from_usize(value: usize) -> Option<Self>;
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

impl Index for u32 {
    const EMPTY: Self = 0;

    type Indexes<P> = Vec<TreeIndex<Self, P>> where P: Copy;
    type Length = usize;

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
    fn saturating_sub(self, other: Self) -> Self {
        u32::saturating_sub(self, other)
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        u32::try_from(value).ok()
    }
}

impl Index for usize {
    const EMPTY: Self = 0;

    type Indexes<P> = Vec<TreeIndex<Self, P>> where P: Copy;
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
    fn saturating_sub(self, other: Self) -> Self {
        usize::saturating_sub(self, other)
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        Some(value)
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
    fn saturating_sub(self, _: Self) -> Self {
        Empty
    }

    #[inline]
    fn from_usize(_: usize) -> Option<Self> {
        Some(Empty)
    }
}

/// A span in the source code, akin to `start..end` so the end of the span is
/// exclusive.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Span<I> {
    /// The start of the span.
    pub start: I,
    /// The end of the span.
    pub end: I,
}

impl<I> Span<I>
where
    I: Index,
{
    /// Construct a new span.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let span = Span::new(4u32, 8u32);
    ///
    /// assert_eq!(span.start, 4);
    /// assert_eq!(span.end, 8);
    /// ```
    #[must_use]
    pub const fn new(start: I, end: I) -> Self {
        Self { start, end }
    }

    /// Construct a span corresponding to the given point.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert_eq!(Span::point(4u32), Span::new(4u32, 4u32));
    /// ```
    #[must_use]
    pub const fn point(at: I) -> Self {
        Self { start: at, end: at }
    }

    /// Join the current span with another.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let a = Span::new(4u32, 8u32);
    /// let b = Span::new(5u32, 9u32);
    ///
    /// let span = a.join(&b);
    ///
    /// assert_eq!(span.start, 4);
    /// assert_eq!(span.end, 9);
    /// assert_eq!(span, b.join(&a));
    /// ```
    #[must_use]
    #[inline]
    pub fn join(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Coerce into a [`ops::Range`] which is useful for slicing.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let a = Span::new(4u32, 8u32);
    ///
    /// assert_eq!(a.range(), 4..8);
    /// ```
    #[must_use]
    pub fn range(self) -> ops::Range<usize> {
        self.start.as_usize()..self.end.as_usize()
    }

    /// The length of the span.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert_eq!(Span::new(0u32, 0u32).len(), 0);
    /// assert_eq!(Span::new(0u32, 10u32).len(), 10);
    /// ```
    #[must_use]
    #[inline]
    pub fn len(&self) -> I::Length {
        self.start.len_to(self.end)
    }

    /// Test if the span is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert!(Span::new(0u32, 0u32).is_empty());
    /// assert!(!Span::new(0u32, 10u32).is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }

    /// Test if span contains the given index.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert!(!Span::new(2u32, 2u32).contains(2));
    /// assert!(Span::new(2u32, 3u32).contains(2));
    /// assert!(!Span::new(2u32, 3u32).contains(3));
    /// ```
    #[must_use]
    #[inline]
    pub fn contains(self, index: I) -> bool {
        self.start <= index && index < self.end
    }
}

impl<I> fmt::Display for Span<I>
where
    I: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl<I> fmt::Debug for Span<I>
where
    I: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (&self.start, &self.end).fmt(f)
    }
}

impl<I> PartialEq<&Span<I>> for Span<I>
where
    I: PartialEq,
{
    #[inline]
    fn eq(&self, other: &&Span<I>) -> bool {
        *self == **other
    }
}

impl<I> PartialEq<Span<I>> for &Span<I>
where
    I: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Span<I>) -> bool {
        **self == *other
    }
}

#[doc(hidden)]
pub trait Indexes<I, P>: self::sealed::Sealed {
    const EMPTY: Self;

    #[doc(hidden)]
    fn push(&mut self, cursor: I, id: P);

    #[doc(hidden)]
    fn binary_search(&self, index: I) -> Result<usize, usize>;

    #[doc(hidden)]
    fn get(&self, index: usize) -> Option<P>;
}

#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct TreeIndex<I, P> {
    pub(crate) index: I,
    pub(crate) id: P,
}

impl From<Empty> for usize {
    #[inline]
    fn from(Empty: Empty) -> Self {
        0
    }
}

impl<I, P> Indexes<I, P> for Vec<TreeIndex<I, P>>
where
    I: Index,
    P: Copy,
{
    const EMPTY: Self = Self::new();

    #[inline]
    fn push(&mut self, index: I, id: P) {
        Vec::push(self, TreeIndex { index, id })
    }

    #[inline]
    fn binary_search(&self, index: I) -> Result<usize, usize> {
        self.binary_search_by(|f| f.index.cmp(&index))
    }

    #[inline]
    fn get(&self, index: usize) -> Option<P> {
        Some(<[_]>::get(self, index)?.id)
    }
}

/// The empty span implementation.
///
/// This can be used in combination with [`Builder::new_with`].
///
/// [`Builder::new_with`]: crate::Builder::new_with
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Empty;

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
