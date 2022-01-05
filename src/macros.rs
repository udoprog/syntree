/// Helper macro for building a tree in place.
///
/// # Examples
///
/// ```
/// use syntree::TreeBuilder;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Syntax {
///     Root,
///     Number,
///     Lit,
///     Whitespace,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut b = TreeBuilder::new();
///
/// b.open(Syntax::Root);
///
/// b.open(Syntax::Number);
/// b.token(Syntax::Lit, 1);
/// b.close()?;
///
/// b.token(Syntax::Whitespace, 3);
///
/// b.open(Syntax::Number);
/// b.token(Syntax::Lit, 2);
/// b.token(Syntax::Lit, 2);
/// b.close()?;
///
/// b.close()?;
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
        $b.open($expr);
        $b.close()?;
    }};

    (@o $b:ident, $expr:expr, $($rest:tt)*) => {{
        $b.open($expr);
        $b.close()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* } $(,)?) => {{
        $b.open($expr);
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* }, $($rest:tt)*) => {{
        $b.open($expr);
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    ($($tt:tt)*) => {{
        let mut b = $crate::TreeBuilder::new();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}
