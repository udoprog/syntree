//! Types associated with performing immutable editing of a tree.

use core::marker::PhantomData;

use alloc::vec::Vec;

use std::collections::HashMap;

use crate::builder::Builder;
use crate::error::Error;
use crate::flavor::Flavor;
use crate::node::{Children, Element, NodeOrToken};
use crate::tree::Tree;

#[derive(Debug)]
pub(crate) enum Change {
    /// Delete the given node or token.
    Delete,
}

/// A recorded set of tree modifications.
///
/// You can use [`ChangeSet::modify`] to construct a new modified tree from an
/// existing one.
///
/// Node ids and token ids are separate index spaces, so changes are recorded
/// against an [`Element`] with [`ChangeSet::remove`], or against an id with
/// [`ChangeSet::remove_node`] or [`ChangeSet::remove_token`].
///
/// # Examples
///
/// ```
/// use syntree::edit::ChangeSet;
///
/// let tree = syntree::tree! {
///     "root" => {
///         "child" => {
///             ("lit", 1),
///             ("lit", 2),
///         },
///         ("whitespace", 3),
///     }
/// };
///
/// let child = tree.first().and_then(|n| n.first()).ok_or("missing child")?;
///
/// let mut change_set = ChangeSet::new();
/// change_set.remove(child);
///
/// assert_eq!(
///     change_set.modify(&tree)?,
///     syntree::tree! {
///         "root" => {
///             ("whitespace", 3)
///         }
///     }
/// );
///
/// let lit = child.first().ok_or("missing lit")?;
///
/// let mut change_set = ChangeSet::new();
/// change_set.remove(lit);
///
/// assert_eq!(
///     change_set.modify(&tree)?,
///     syntree::tree! {
///         "root" => {
///             "child" => {
///                 ("lit", 2),
///             },
///             ("whitespace", 3)
///         }
///     }
/// );
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub struct ChangeSet<T, F>
where
    T: Copy,
    F: Flavor,
{
    nodes: HashMap<F::Pointer, Change>,
    tokens: HashMap<F::Pointer, Change>,
    _marker: PhantomData<T>,
}

impl<T, F> ChangeSet<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new empty [`ChangeSet`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, F> ChangeSet<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Register the removal of a node or token in the changeset. Removing a
    /// node removes everything inside of it. Only one kind of modification
    /// for a given element will be preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::edit::ChangeSet;
    ///
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("lit", 1),
    ///             ("lit", 2),
    ///         },
    ///         ("whitespace", 3),
    ///     }
    /// };
    ///
    /// let child = tree.first().and_then(|n| n.first()).ok_or("missing child")?;
    ///
    /// let mut change_set = ChangeSet::new();
    /// change_set.remove(child);
    ///
    /// assert_eq!(
    ///     change_set.modify(&tree)?,
    ///     syntree::tree! {
    ///         "root" => {
    ///             ("whitespace", 3)
    ///         }
    ///     }
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn remove(&mut self, element: Element<'_, T, F>) {
        match element {
            NodeOrToken::Node(node) => self.remove_node(node.id()),
            NodeOrToken::Token(token) => self.remove_token(token.id()),
        }
    }

    /// Register the removal of the interior node with the given
    /// [id][crate::Node::id], and everything inside of it.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::edit::ChangeSet;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("root")?;
    /// let child = tree.open("child")?;
    /// tree.token("lit", 1)?;
    /// tree.close()?;
    /// tree.token("whitespace", 3)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let mut change_set = ChangeSet::new();
    /// change_set.remove_node(child);
    ///
    /// assert_eq!(
    ///     change_set.modify(&tree)?,
    ///     syntree::tree! {
    ///         "root" => {
    ///             ("whitespace", 3)
    ///         }
    ///     }
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn remove_node(&mut self, id: F::Pointer) {
        self.nodes.insert(id, Change::Delete);
    }

    /// Register the removal of the token with the given
    /// [id][crate::node::Token::id].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::edit::ChangeSet;
    ///
    /// let mut tree = syntree::Builder::new();
    ///
    /// tree.open("root")?;
    /// tree.token("lit", 1)?;
    /// let ws = tree.token("whitespace", 3)?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let mut change_set = ChangeSet::new();
    /// change_set.remove_token(ws);
    ///
    /// assert_eq!(
    ///     change_set.modify(&tree)?,
    ///     syntree::tree! {
    ///         "root" => {
    ///             ("lit", 1)
    ///         }
    ///     }
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn remove_token(&mut self, id: F::Pointer) {
        self.tokens.insert(id, Change::Delete);
    }

    fn is_deleted(&self, element: &Element<'_, T, F>) -> bool {
        let change = match element {
            NodeOrToken::Node(node) => self.nodes.get(&node.id()),
            NodeOrToken::Token(token) => self.tokens.get(&token.id()),
        };

        matches!(change, Some(Change::Delete))
    }

    /// Construct a modified tree where the recorded modifications have been
    /// applied.
    ///
    /// Spans are compacted: the tokens which remain are laid out
    /// back-to-back from offset zero, so removed tokens and gaps left by
    /// explicit spans do not leave holes. Interior nodes which are left
    /// without children are kept as empty nodes.
    ///
    /// # Errors
    ///
    /// Errors with [`Error::Overflow`] in case we run out of node
    /// identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::edit::ChangeSet;
    ///
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("lit", 1),
    ///             ("lit", 2),
    ///         },
    ///         ("whitespace", 3),
    ///     }
    /// };
    ///
    /// let child = tree.first().and_then(|n| n.first()).ok_or("missing child")?;
    /// let mut change_set = ChangeSet::new();
    /// change_set.remove(child);
    ///
    /// assert_eq!(
    ///     change_set.modify(&tree)?,
    ///     syntree::tree! {
    ///         "root" => {
    ///             ("whitespace", 3)
    ///         }
    ///     }
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn modify(&mut self, tree: &Tree<T, F>) -> Result<Tree<T, F>, Error<F::Error>> {
        let (nodes, tokens) = tree.counts();
        let mut output = Builder::<T, F>::with_capacities(nodes, tokens)?;

        // Stepping a `Children` iterator is constant time, unlike stepping
        // from a token with `Token::next`.
        let mut stack = Vec::<Children<'_, T, F>>::new();
        stack.push(tree.children().with_trivia());

        while let Some(children) = stack.last_mut() {
            let Some(element) = children.next() else {
                stack.pop();

                // The root level has no node to close.
                if !stack.is_empty() {
                    output.close()?;
                }

                continue;
            };

            if self.is_deleted(&element) {
                continue;
            }

            match element {
                NodeOrToken::Node(node) => {
                    output.open(node.value())?;
                    stack.push(node.children().with_trivia());
                }
                NodeOrToken::Token(token) if token.is_trivia() => {
                    output.trivia(token.value(), token.span().len())?;
                }
                NodeOrToken::Token(token) => {
                    output.token(token.value(), token.span().len())?;
                }
            }
        }

        output.build()
    }
}

impl<T, F> Default for ChangeSet<T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            tokens: HashMap::new(),
            _marker: PhantomData,
        }
    }
}
