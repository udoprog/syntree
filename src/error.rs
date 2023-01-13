use core::fmt;

use crate::Id;

/// Errors raised while building a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
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
    /// let result = tree.close_at(&c, "operation");
    /// assert_eq!(result, Err(Error::CloseAtError));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    CloseAtError,
    /// Numerical overflow.
    ///
    /// This only happens under extreme circumstances or if a feature is enabled
    /// which narrows the width of an identifier to the degree that this error
    /// is easier to accomplish.
    Overflow,
    /// The node of the given id is missing.
    MissingNode(Id),
    /// Missing next.
    MissingCloseAtLinksNext,
    /// Missing sibling.
    MissingCloseAtSibling,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
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
            Error::MissingNode(id) => {
                write!(f, "missing node with id `{}`", id.0.get())
            }
            Error::MissingCloseAtLinksNext => {
                write!(f, "missing links next while closing checkpoint")
            }
            Error::MissingCloseAtSibling => {
                write!(f, "missing current sibling while closing checkpoint")
            }
        }
    }
}
