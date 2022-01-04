/// Helper macro for building a tree in place.
///
/// # Examples
///
/// ```
/// use syntree::{Span, TreeBuilder};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Syntax {
///     Root,
///     Number,
///     Lit,
///     Whitespace,
/// }
///
/// # fn main() -> anyhow::Result<()> {
/// let mut b = TreeBuilder::new();
///
/// b.start_node(Syntax::Root);
///
/// b.start_node(Syntax::Number);
/// b.token(Syntax::Lit, Span::new(1, 2));
/// b.end_node()?;
///
/// b.token(Syntax::Whitespace, Span::new(2, 5));
///
/// b.start_node(Syntax::Number);
/// b.token(Syntax::Lit, Span::new(5, 7));
/// b.token(Syntax::Lit, Span::new(7, 9));
/// b.end_node()?;
///
/// b.end_node()?;
///
/// let tree = b.build()?;
///
/// let expected = syntree::tree! {
///     >> Syntax::Root,
///         >> Syntax::Number,
///             + (1, 2) Syntax::Lit,
///         <<
///         + (2, 5) Syntax::Whitespace,
///         >> Syntax::Number,
///             + (5, 7) Syntax::Lit,
///             + (7, 9) Syntax::Lit,
///         <<
///     <<
/// };
///
/// assert_eq!(expected, tree);
/// # Ok(()) }
/// ```
#[macro_export]
macro_rules! tree {
    (@o $b:ident, == $expr:expr, $($tt:tt)*) => {{
        $b.start_node($expr);
        $b.end_node()?;
        $crate::tree!(@o $b, $($tt)*);
    }};

    (@o $b:ident, >> $expr:expr, $($tt:tt)*) => {{
        $b.start_node($expr);
        $crate::tree!(@o $b, $($tt)*);
    }};

    (@o $b:ident, << $(,)? $($tt:tt)*) => {{
        $b.end_node()?;
        $crate::tree!(@o $b, $($tt)*);
    }};

    (@o $b:ident, + ($start:expr, $end:expr) $expr:expr, $($tt:tt)*) => {{
        $b.token($expr, $crate::Span::new($start, $end));
        $crate::tree!(@o $b, $($tt)*);
    }};

    (@o $b:ident, $(,)?) => {};

    ($($tt:tt)*) => {{
        let mut b = $crate::TreeBuilder::new();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}
