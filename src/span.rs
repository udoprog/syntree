use std::fmt;

/// A span in the source code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Span {
    /// The start of the span.
    pub start: u32,
    /// The end of the span.
    pub end: u32,
}

impl Span {
    /// Construct a new span.
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let span = Span::new(4, 8);
    ///
    /// assert_eq!(span.start, 4);
    /// assert_eq!(span.end, 8);
    /// ```
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.try_into().expect("start index out of bounds"),
            end: end.try_into().expect("end index out of bounds"),
        }
    }

    /// Join the current span with another.
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
    pub fn join(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
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
