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
/// b.token(Syntax::Lit, 1);
/// b.end_node()?;
///
/// b.token(Syntax::Whitespace, 3);
///
/// b.start_node(Syntax::Number);
/// b.token(Syntax::Lit, 2);
/// b.token(Syntax::Lit, 2);
/// b.end_node()?;
///
/// b.end_node()?;
///
/// let tree = b.build()?;
///
/// let expected = syntree::tree! {
///     Syntax::Root => {
///         Syntax::Number => {
///             (Syntax::Lit, 1)
///         },
///         (Syntax::Whitespace, 3),
///         Syntax::Number => {
///             (Syntax::Lit, 2),
///             (Syntax::Lit, 2)
///         }
///     }
/// };
///
/// assert_eq!(expected, tree);
/// # Ok(()) }
/// ```
#[macro_export]
macro_rules! tree {
    (@o $b:ident,) => {};

    (@o $b:ident, ($expr:expr, $len:expr) $(,)?) => {{
        $b.token($expr, $len);
    }};

    (@o $b:ident, ($expr:expr, $len:expr), $($rest:tt)*) => {{
        $b.token($expr, $len);
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr $(,)?) => {{
        $b.start_node($expr);
        $b.end_node()?;
    }};

    (@o $b:ident, $expr:expr, $($rest:tt)*) => {{
        $b.start_node($expr);
        $b.end_node()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* } $(,)?) => {{
        $b.start_node($expr);
        $crate::tree!(@o $b, $($tt)*);
        $b.end_node()?;
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* }, $($rest:tt)*) => {{
        $b.start_node($expr);
        $crate::tree!(@o $b, $($tt)*);
        $b.end_node()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    ($($tt:tt)*) => {{
        let mut b = $crate::TreeBuilder::new();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}
