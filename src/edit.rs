//! Types associated with performing immutable editing of a tree.

use core::marker::PhantomData;

use alloc::vec::Vec;

use std::collections::HashMap;

use crate::builder::Builder;
use crate::error::Error;
use crate::flavor::Flavor;
use crate::node::Node;
use crate::tree::Tree;

#[derive(Debug)]
pub(crate) enum Change {
    /// Delete the given node.
    Delete,
}

/// A recorded set of tree modifications.
///
/// You can use [`ChangeSet::modify`] to construct a new modified tree from an
/// existing one.
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
/// change_set.remove(child.id());
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
/// change_set.remove(lit.id());
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
    changes: HashMap<F::Pointer, Change>,
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
    /// Register a node removal in the changeset. Only one kind of modification
    /// for a given node will be preserved.
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
    /// change_set.remove(child.id());
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
    pub fn remove(&mut self, id: F::Pointer) {
        self.changes.insert(id, Change::Delete);
    }

    /// Construct a modified tree where the recorded modifications have been
    /// applied.
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
    /// change_set.remove(child.id());
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
        let mut output = Builder::<T, F>::with_capacity(tree.capacity())?;
        let mut parents = Vec::<Node<'_, T, F>>::new();
        let mut current = tree.first();

        loop {
            let Some(node) = current else {
                let Some(parent) = parents.pop() else {
                    break;
                };

                output.close()?;
                current = parent.next();
                continue;
            };

            if let Some(Change::Delete) = self.changes.get(&node.id()) {
                current = node.next();
                continue;
            }

            if let Some(first) = node.first() {
                output.open(node.value())?;
                parents.push(node);
                current = Some(first);
            } else {
                // Leaves are laid out back-to-back, so removed spans are
                // compacted away.
                output.token(node.value(), node.span().len())?;
                current = node.next();
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
            changes: HashMap::new(),
            _marker: PhantomData,
        }
    }
}
