use std::error::Error;
use std::fmt;

/// Errors raised while building a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreeError {
    /// Error raised by [TreeBuilder::close][crate::TreeBuilder::close] if there
    /// currently is no node being built.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{TreeBuilder, TreeError};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("root")?;
    /// tree.close()?;
    ///
    /// // Syntax::Root and Syntax::Child is left open.
    /// assert_eq!(tree.close(), Err(TreeError::CloseError));
    /// # Ok(()) }
    /// ```
    CloseError,
    /// Error raised by [TreeBuilder::build][crate::TreeBuilder::build] if the
    /// tree isn't correctly balanced.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{TreeBuilder, TreeError};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// tree.open("number")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    ///
    /// tree.open("number")?;
    ///
    /// // Syntax::Number is left open.
    /// assert_eq!(tree.build(), Err(TreeError::BuildError));
    /// # Ok(()) }
    /// ```
    BuildError,
    /// Error raised by [TreeBuilder::close_at][crate::TreeBuilder::close_at] if
    /// we're not trying to close at a sibling node.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::{TreeBuilder, TreeError};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
    ///
    /// let c = tree.checkpoint()?;
    ///
    /// tree.open("child")?;
    /// tree.token("token", 3)?;
    ///
    /// let result = tree.close_at(c, "operation");
    /// assert_eq!(result, Err(TreeError::CloseAtError));
    /// # Ok(()) }
    /// ```
    CloseAtError,
    /// Cursor overflowed.
    ///
    /// This only happens under extreme circumstances or if a feature is enabled
    /// which narrows the width of an identifier to the degree that this error
    /// is easier to accomplish.
    CursorOverflow,
    /// Identifier overflowed.
    ///
    /// This only happens under extreme circumstances or if a feature is enabled
    /// which narrows the width of an identifier to the degree that this error
    /// is easier to accomplish.
    IdOverflow,
}

impl Error for TreeError {}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::CloseError => {
                write!(f, "no node being built")
            }
            TreeError::BuildError => {
                write!(f, "tree is currently being built")
            }
            TreeError::CloseAtError => {
                write!(
                    f,
                    "trying to close a node which is not a sibling of the checkpoint being closed"
                )
            }
            TreeError::CursorOverflow => {
                write!(f, "index cursor overflow")
            }
            TreeError::IdOverflow => {
                write!(f, "identifier overflow")
            }
        }
    }
}
