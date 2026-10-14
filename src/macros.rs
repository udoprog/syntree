/// Helper macro for building a tree in place.
///
/// The tree is built with [`Builder::new`], so it uses the default flavor. Use
/// [`tree_with!`] to build a tree of another flavor.
///
/// Each element is one of:
/// * `value` a token with an empty span, see [`Builder::token_empty`].
/// * `(value, len)` a token of length `len`, see [`Builder::token`].
/// * `(value, (start, end))` a token with an explicit span, see
///   [`Builder::token_with`].
/// * `value => { .. }` a node whose span covers its children, see
///   [`Builder::open`].
/// * `(value, (start, end)) => { .. }` a node with an explicit span, see
///   [`Builder::open_with`].
///
/// Elements are separated by commas. The macro expands to builder calls
/// followed by `?`, so it can only be used in a function (or doctest) returning
/// a `Result` whose error type implements `From<Error<F::Error>>`, where `F` is
/// the flavor of the tree.
///
/// [`Builder::new`]: crate::Builder::new
/// [`Builder::token_empty`]: crate::Builder::token_empty
/// [`Builder::token`]: crate::Builder::token
/// [`Builder::token_with`]: crate::Builder::token_with
/// [`Builder::open`]: crate::Builder::open
/// [`Builder::open_with`]: crate::Builder::open_with
/// [`tree_with!`]: crate::tree_with
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
/// # Ok::<_,  Box<dyn core::error::Error>>(())
/// ```
///
/// Explicit spans:
///
/// ```
/// use syntree::Span;
///
/// let tree = syntree::tree! {
///     ("root", (0, 10)) => {
///         ("token", (2, 5)),
///     }
/// };
///
/// let root = tree.first().ok_or("missing root")?;
/// assert_eq!(root.span(), Span::new(0, 10));
///
/// let token = root.first().ok_or("missing token")?;
/// assert_eq!(token.span(), Span::new(2, 5));
/// # Ok::<_,  Box<dyn core::error::Error>>(())
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

/// Helper macro for building a tree in place with a custom flavor.
///
/// This takes the same input as [`tree!`], but builds the tree with
/// [`Builder::new_with`]. The flavor is inferred from the type the result is
/// used as, so it usually needs a type annotation such as
/// `let tree: Tree<_, MyFlavor> = ..`.
///
/// Like [`tree!`], it expands to builder calls followed by `?`, so it can only
/// be used in a function returning a `Result` whose error type implements
/// `From<Error<F::Error>>`, where `F` is the flavor of the tree.
///
/// [`tree!`]: crate::tree
/// [`Builder::new_with`]: crate::Builder::new_with
///
/// # Examples
///
/// ```
/// use syntree::{Empty, EmptyVec, Tree, TreeIndex};
///
/// syntree::flavor! {
///     struct FlavorEmpty {
///         type Index = Empty;
///         type Indexes = EmptyVec<TreeIndex<Self>>;
///     }
/// };
///
/// let tree: Tree<_, FlavorEmpty> = syntree::tree_with! {
///     "root" => {
///         "child" => {
///             "token"
///         },
///         "child2"
///     }
/// };
///
/// let expected: Tree<_, FlavorEmpty> = syntree::tree_with! {
///     "root" => {
///         "child" => {
///             ("token", Empty)
///         },
///         "child2"
///     }
/// };
///
/// assert_eq!(tree, expected);
/// # Ok::<_,  Box<dyn core::error::Error>>(())
/// ```
#[macro_export]
macro_rules! tree_with {
    ($($tt:tt)*) => {{
        let mut b = $crate::Builder::new_with();
        $crate::tree!(@o b, $($tt)*);
        b.build()?
    }};
}
