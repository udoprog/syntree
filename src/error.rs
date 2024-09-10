use core::convert::Infallible;
use core::fmt;

/// Errors raised while building a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error<E = Infallible> {
    /// Error raised by [Builder::close][crate::Builder::close] if there
    /// currently is no node being built.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Builder, Error};
    ///
    /// let mut tree = Builder::new();
    ///
    /// tree.open("root")?;
    /// tree.close()?;
    ///
    /// // Syntax::Root and Syntax::Child is left open.
    /// assert_eq!(tree.close(), Err(Error::CloseError));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    CloseError,
    /// Error raised by [Builder::build][crate::Builder::build] if the
    /// tree isn't correctly balanced.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Builder, Error};
    ///
    /// let mut tree = Builder::new();
    ///
    /// tree.open("number")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    ///
    /// tree.open("number")?;
    ///
    /// // Syntax::Number is left open.
    /// assert_eq!(tree.build(), Err(Error::BuildError));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    BuildError,
    /// Error raised by [Builder::close_at][crate::Builder::close_at] if
    /// we're not trying to close at a sibling node.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{Builder, Error};
    ///
    /// let mut tree = Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    ///
    /// tree.open("child")?;
    /// tree.token("token", 3)?;
    ///
    /// assert_eq!(tree.close_at(&c, "operation"), Err(Error::CloseAtError));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    CloseAtError,
    /// Numerical overflow.
    ///
    /// This only happens under extreme circumstances or if a feature is enabled
    /// which narrows the width of an identifier to the degree that this error
    /// is easier to accomplish.
    ///
    /// # Examples
    ///
    /// This is an example where we're trying to build a really small tree using
    /// u8 pointers:
    ///
    /// ```
    /// use syntree::{Builder, Error};
    ///
    /// syntree::flavor! {
    ///     struct CustomFlavor {
    ///         type Index = u32;
    ///         type Width = u8;
    ///     }
    /// }
    ///
    /// let mut tree: Builder<_, CustomFlavor> = Builder::new_with();
    ///
    /// for d in 0..u8::MAX {
    ///     tree.token(d, 1)?;
    /// }
    ///
    /// assert_eq!(tree.token(255, 1), Err(Error::Overflow));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    Overflow,
    /// The node of the given id is missing.
    ///
    /// # Examples
    ///
    /// The following showcases what could happen if you mix checkpoints from
    /// two compatible trees:
    ///
    /// ```
    /// use syntree::{Builder, Error};
    ///
    /// let mut a = Builder::new();
    /// let mut b = Builder::new();
    ///
    /// b.open("child")?;
    /// b.close()?;
    ///
    /// let c = b.checkpoint()?;
    ///
    /// assert_eq!(a.close_at(&c, "operation"), Err(Error::MissingNode(0)));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    MissingNode(usize),
    /// An error raised by the particular [Flavor] in use.
    ///
    /// [Flavor]: crate::Flavor
    Flavor(E),
}

impl<E> From<E> for Error<E> {
    #[inline]
    fn from(error: E) -> Self {
        Error::Flavor(error)
    }
}

impl<E> core::error::Error for Error<E>
where
    E: 'static + core::error::Error,
{
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Flavor(error) => Some(error),
            _ => None,
        }
    }
}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CloseError => {
                write!(f, "no node being built")
            }
            Error::BuildError => {
                write!(f, "tree is currently being built")
            }
            Error::CloseAtError => {
                write!(
                    f,
                    "trying to close a node which is not a sibling of the checkpoint being closed"
                )
            }
            Error::Overflow => {
                write!(f, "numerical overflow")
            }
            Error::MissingNode(p) => {
                write!(f, "missing node with id `{p}`")
            }
            Error::Flavor(error) => error.fmt(f),
        }
    }
}
