use std::fmt;
use std::mem::size_of;
use std::ops;

/// The index used in a span.
#[cfg(syntree_compact)]
pub(crate) type Index = u32;

#[cfg(syntree_compact)]
#[inline]
pub(crate) fn usize_to_index(value: usize) -> Option<u32> {
    u32::try_from(value).ok()
}

#[cfg(not(syntree_compact))]
pub(crate) type Index = usize;

#[cfg(not(syntree_compact))]
#[inline]
pub(crate) fn usize_to_index(value: usize) -> Option<Index> {
    Some(value)
}

/// Ensure that the specified index is smaller or equal to [usize].
const _: () = assert!(size_of::<Index>() <= size_of::<usize>());

/// A span in the source code, akin to `start..end` so the end of the span is
/// exclusive.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Span {
    /// The start of the span.
    pub start: Index,
    /// The end of the span.
    pub end: Index,
}

impl Span {
    /// Construct a new span.
    ///
    /// # Panics
    ///
    /// Panics if `start` does not precede or equal to `end.
    ///
    /// ```should_panic
    /// use syntree::Span;
    ///
    /// Span::new(9, 8);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let span = Span::new(4, 8);
    ///
    /// assert_eq!(span.start, 4);
    /// assert_eq!(span.end, 8);
    /// ```
    pub const fn new(start: Index, end: Index) -> Self {
        assert!(start <= end, "start of the span must come before end");
        Self { start, end }
    }

    /// Construct a span corresponding to the given point.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert_eq!(Span::point(4), Span::new(4, 4));
    /// ```
    pub const fn point(at: Index) -> Self {
        Self { start: at, end: at }
    }

    /// Join the current span with another.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let a = Span::new(4, 8);
    /// let b = Span::new(5, 9);
    ///
    /// let span = a.join(&b);
    ///
    /// assert_eq!(span.start, 4);
    /// assert_eq!(span.end, 9);
    /// assert_eq!(span, b.join(&a));
    /// ```
    pub const fn join(&self, other: &Self) -> Self {
        Self {
            start: if self.start < other.start {
                self.start
            } else {
                other.start
            },
            end: if self.end > other.end {
                self.end
            } else {
                other.end
            },
        }
    }

    /// Coerce into a [ops::Range] which is useful for slicing.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let a = Span::new(4, 8);
    ///
    /// assert_eq!(a.range(), 4..8);
    /// ```
    pub const fn range(self) -> ops::Range<usize> {
        (self.start as usize)..(self.end as usize)
    }

    /// The length of the span.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert_eq!(Span::new(0, 0).len(), 0);
    /// assert_eq!(Span::new(0, 10).len(), 10);
    /// ```
    pub const fn len(&self) -> Index {
        self.end.saturating_sub(self.start)
    }

    /// Test if the span is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert!(Span::new(0, 0).is_empty());
    /// ```
    pub const fn is_empty(self) -> bool {
        self.end == self.start
    }

    /// Test if span contains the given index.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// assert!(!Span::new(2, 2).contains(2));
    /// assert!(Span::new(2, 3).contains(2));
    /// assert!(!Span::new(2, 3).contains(3));
    /// ```
    pub const fn contains(self, index: Index) -> bool {
        self.start <= index && index < self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (&self.start, &self.end).fmt(f)
    }
}

impl PartialEq<&Span> for Span {
    #[inline]
    fn eq(&self, other: &&Span) -> bool {
        *self == **other
    }
}

impl PartialEq<Span> for &Span {
    #[inline]
    fn eq(&self, other: &Span) -> bool {
        **self == *other
    }
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Span {}

    impl Sealed for () {}
}

/// Trait governing how to build and adjust spans.
pub trait SpanBuilder: self::sealed::Sealed + Copy {
    /// Empty span.
    const EMPTY: Self;

    /// Construct a span at the given index.
    #[doc(hidden)]
    fn point(index: Index) -> Self;

    /// Construct a new span.
    #[doc(hidden)]
    fn new(start: Index, end: Index) -> Self;

    /// Get start of the span.
    #[doc(hidden)]
    fn start(&self) -> Index;

    /// Get end of the span.
    #[doc(hidden)]
    fn end(&self) -> Index;

    /// Update end value of the span.
    #[doc(hidden)]
    fn set_end(&mut self, end: Index);

    /// Value representing the length of a span.
    #[doc(hidden)]
    fn len(&self) -> usize;
}

impl SpanBuilder for Span {
    const EMPTY: Self = Span::point(0);

    #[inline]
    fn point(index: Index) -> Self {
        Span::point(index)
    }

    #[inline]
    fn new(start: Index, end: Index) -> Self {
        Span::new(start, end)
    }

    #[inline]
    fn start(&self) -> Index {
        self.start
    }

    #[inline]
    fn end(&self) -> Index {
        self.end
    }

    #[inline]
    fn set_end(&mut self, end: Index) {
        self.end = end;
    }

    #[inline]
    fn len(&self) -> usize {
        Span::len(self)
    }
}

impl SpanBuilder for () {
    const EMPTY: Self = ();

    #[inline]
    fn point(_: Index) -> Self {
        ()
    }

    #[inline]
    fn new(_: Index, _: Index) -> Self {
        ()
    }

    #[inline]
    fn start(&self) -> Index {
        0
    }

    #[inline]
    fn end(&self) -> Index {
        0
    }

    #[inline]
    fn set_end(&mut self, _: Index) {}

    #[inline]
    fn len(&self) -> usize {
        0
    }
}
