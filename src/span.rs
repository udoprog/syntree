use std::{fmt, ops};

/// A span in the source code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Span {
    /// The start of the span.
    pub start: usize,
    /// The end of the span.
    pub end: usize,
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
    pub const fn new(start: usize, end: usize) -> Self {
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
    pub const fn point(at: usize) -> Self {
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
    /// let span = a.join(b);
    ///
    /// assert_eq!(span.start, 4);
    /// assert_eq!(span.end, 9);
    /// assert_eq!(span, b.join(a));
    /// ```
    pub const fn join(self, other: Self) -> Self {
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
        self.start..self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Span")
            .field(&self.start)
            .field(&self.end)
            .finish()
    }
}
