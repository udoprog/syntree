use core::fmt;
use core::ops;

use crate::index::Index;

/// A span in the source code, akin to `start..end` so the end of the span is
/// exclusive.
#[derive(Clone, Copy, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Span<I> {
    /// The start of the span.
    pub start: I,
    /// The end of the span.
    pub end: I,
}

impl<A, B> PartialEq<Span<A>> for Span<B>
where
    B: PartialEq<A>,
{
    fn eq(&self, other: &Span<A>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<I> Span<I> {
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
    pub const fn point(at: I) -> Self
    where
        I: Copy,
    {
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
    pub fn join(&self, other: &Self) -> Self
    where
        I: Copy + Ord,
    {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
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
    pub fn is_empty(&self) -> bool
    where
        I: Eq,
    {
        self.end == self.start
    }

    /// Test if span contains the given index.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert!(!Span::new(2u32, 2u32).contains(&2));
    /// assert!(Span::new(2u32, 3u32).contains(&2));
    /// assert!(!Span::new(2u32, 3u32).contains(&3));
    /// ```
    #[must_use]
    #[inline]
    pub fn contains<U>(self, index: &U) -> bool
    where
        I: PartialOrd<U>,
        U: PartialOrd<I> + ?Sized,
    {
        &self.start <= index && index < &self.end
    }
}

impl<I> Span<I>
where
    I: Index,
{
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
