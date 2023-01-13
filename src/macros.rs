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
/// let mut tree = TreeBuilder::new();
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
/// tree.token(Syntax::Lit, 2)?;
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
///             (Syntax::Lit, 2)
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

    (@o $b:ident, ($expr:expr, $len:expr) $(,)?) => {{
        $b.token($expr, $len)?;
    }};

    (@o $b:ident, ($expr:expr, $len:expr), $($rest:tt)*) => {{
        $b.token($expr, $len)?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr $(,)?) => {{
        $b.open($expr)?;
        $b.close()?;
    }};

    (@o $b:ident, $expr:expr, $($rest:tt)*) => {{
        $b.open($expr)?;
        $b.close()?;
        $crate::tree!(@o $b, $($rest)*);
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* } $(,)?) => {{
        $b.open($expr)?;
        $crate::tree!(@o $b, $($tt)*);
        $b.close()?;
    }};

    (@o $b:ident, $expr:expr => { $($tt:tt)* }, $($rest:tt)*) => {{
        $b.open($expr)?;
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

/// Helper macro for building a tree in place with a custom span.
///
/// # Examples
///
/// ```
/// use syntree::Tree;
///
/// let tree: Tree<_, ()> = syntree::tree_with! {
///     "root" => {
///         "child" => {
///             ("token", ())
///         },
///         "child2"
///     }
/// };
/// # Ok::<_,  Box<dyn std::error::Error>>(())
/// ```
#[macro_export]
macro_rules! tree_with {
    ($($tt:tt)*) => {{
        let mut b = $crate::TreeBuilder::new_with();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}
