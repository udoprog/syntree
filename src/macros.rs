/// Helper macro for building a tree in place.
///
/// # Examples
///
/// ```
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Syntax {
///     Root,
///     Number,
///     Lit,
///     Whitespace,
/// }
///
/// let mut tree = syntree::Builder::new();
///
/// tree.open(Syntax::Root)?;
///
/// tree.open(Syntax::Number)?;
/// tree.token(Syntax::Lit, 1)?;
/// tree.close()?;
///
/// tree.token(Syntax::Whitespace, 3)?;
///
/// tree.open(Syntax::Number)?;
/// tree.token(Syntax::Lit, 2)?;
/// tree.token_empty(Syntax::Lit)?;
/// tree.close()?;
///
/// tree.close()?;
///
/// let tree = tree.build()?;
///
/// let expected = syntree::tree! {
///     Syntax::Root => {
///         Syntax::Number => {
///             (Syntax::Lit, 1)
///         },
///         (Syntax::Whitespace, 3),
///         Syntax::Number => {
///             (Syntax::Lit, 2),
///             Syntax::Lit
///         }
///     }
/// };
///
/// assert_eq!(expected, tree);
/// # Ok::<_,  Box<dyn std::error::Error>>(())
/// ```
#[macro_export]
macro_rules! tree {
    (@o $b:ident,) => {};

    (@o $b:ident, ($expr:expr, ($start:expr, $end:expr)) $(,)?) => {{
        $b.token_with($expr, $crate::Span::new($start, $end))?;
    }};

    (@o $b:ident, ($expr:expr, $len:expr) $(,)?) => {{
        $b.token($expr, $len)?;
    }};

    (@o $b:ident, ($expr:expr, ($start:expr, $end:expr)), $($rest:tt)*) => {{
        $b.token_with($expr, $crate::Span::new($start, $end))?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, ($expr:expr, $len:expr), $($rest:tt)*) => {{
        $b.token($expr, $len)?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr $(,)?) => {{
        $b.token_empty($expr)?;
    }};

    (@o $b:ident, $expr:expr, $($rest:tt)*) => {{
        $b.token_empty($expr)?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, ($expr:expr, ($start:expr, $end:expr)) => { $($tt:tt)* } $(,)?) => {{
        $b.open_with($expr, $crate::Span::new($start, $end))?;
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* } $(,)?) => {{
        $b.open($expr)?;
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
    }};

    (@o $b:ident, ($expr:expr, ($start:expr, $end:expr)) => { $($tt:tt)* }, $($rest:tt)*) => {{
        $b.open_with($expr, $crate::Span::new($start, $end))?;
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* }, $($rest:tt)*) => {{
        $b.open($expr)?;
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    ($($tt:tt)*) => {{
        let mut b = $crate::Builder::new();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}

/// Helper macro for building a tree in place with a custom span.
///
/// # Examples
///
/// ```
/// use syntree::{Empty, Tree};
///
/// let tree: Tree<_, Empty, usize> = syntree::tree_with! {
///     "root" => {
///         "child" => {
///             "token"
///         },
///         "child2"
///     }
/// };
///
/// let expected: Tree<_, Empty, u32> = syntree::tree_with! {
///     "root" => {
///         "child" => {
///             ("token", Empty)
///         },
///         "child2"
///     }
/// };
///
/// assert_eq!(tree, expected);
/// # Ok::<_,  Box<dyn std::error::Error>>(())
/// ```
#[macro_export]
macro_rules! tree_with {
    ($($tt:tt)*) => {{
        let mut b = $crate::Builder::new_with();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}
