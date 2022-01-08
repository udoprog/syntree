use std::collections::HashMap;

use crate::links::Links;
use crate::non_max::NonMax;
use crate::span::Index;
use crate::{Id, Kind, Node, Span, Tree, TreeError};

#[derive(Debug)]
pub(crate) enum Change {
    /// Delete the given node.
    Delete,
}

/// A recorded set of tree modifications.
///
/// You can use [ChangeSet::modify] to construct a new modified tree from an
/// existing one.
///
/// # Examples
///
/// ```
/// use syntree::ChangeSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// # Ok(()) }
/// ```
pub struct ChangeSet<T> {
    changes: HashMap<NonMax, Change>,
    #[allow(unused)]
    trees: Vec<Tree<T>>,
}

impl<T> ChangeSet<T> {
    /// Construct a new empty [ChangeSet].
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a node removal in the changeset. Only one kind of modification
    /// for a given node will be preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::ChangeSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// # Ok(()) }
    /// ```
    pub fn remove(&mut self, id: Id) {
        self.changes.insert(id.0, Change::Delete);
    }

    /// Construct a modified tree where the recorded modifications have been
    /// applied.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::ChangeSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// # Ok(()) }
    /// ```
    pub fn modify(&mut self, tree: &Tree<T>) -> Result<Tree<T>, TreeError>
    where
        T: Clone,
    {
        let mut output = Tree::with_capacity(tree.capacity());

        let mut refactor = RefactorWalk {
            parents: Vec::new(),
            prev: None,
        };

        let mut cursor = Index::default();

        // The specified sub-tree depth is being deleted.
        let mut current = tree.first().map(|node| (node, false));

        while let Some((mut node, mut first)) = current.take() {
            let node_id = NonMax::new(output.len()).ok_or(TreeError::Overflow)?;

            if let Some(change) = self.changes.get(&node_id) {
                match change {
                    Change::Delete => {
                        let skipped = match refactor.skip_subtree(node, first) {
                            Some(output) => output,
                            None => continue,
                        };

                        node = skipped.node;
                        first = skipped.first;
                    }
                }
            }

            if refactor.parents.is_empty() {
                let (first, last) = output.links_mut();

                if first.is_none() {
                    *first = Some(node_id);
                }

                *last = Some(node_id);
            }

            // Since we are the first node in the sequence we're obligated to
            // set the first child of the parent.
            let prev = if !first {
                let prev = refactor.prev.take();

                if let Some(prev) = prev.and_then(|id| output.get_mut(id)) {
                    prev.next = Some(node_id);
                }

                prev
            } else {
                None
            };

            let span = match node.kind() {
                Kind::Node => Span::point(cursor),
                Kind::Token => {
                    let len = node.span().len();

                    if len > 0 {
                        output.push_index(cursor, node_id);
                        let start = cursor;
                        cursor = cursor
                            .checked_add(node.span().len())
                            .ok_or(TreeError::Overflow)?;
                        Span::new(start, cursor)
                    } else {
                        Span::point(cursor)
                    }
                }
            };

            let parent = refactor.parents.last().map(|n| n.1);

            if let Some(parent) = parent.and_then(|id| output.get_mut(id)) {
                if parent.first.is_none() {
                    parent.first = Some(node_id);
                }

                parent.last = Some(node_id);
            }

            output.push(Links {
                data: node.value().clone(),
                kind: node.kind(),
                span,
                parent,
                prev,
                next: None,
                first: None,
                last: None,
            });

            current = refactor.step(node, node_id);
        }

        output.span_mut().end = cursor;
        Ok(output)
    }
}

impl<T> Default for ChangeSet<T> {
    fn default() -> Self {
        Self {
            changes: HashMap::new(),
            trees: Vec::new(),
        }
    }
}

/// The state of the skipped subtree.
struct Skipped<'a, T> {
    node: Node<'a, T>,
    first: bool,
}

struct RefactorWalk<'a, T> {
    parents: Vec<(Node<'a, T>, NonMax)>,
    prev: Option<NonMax>,
}

impl<'a, T> RefactorWalk<'a, T> {
    fn skip_subtree(&mut self, node: Node<'a, T>, first: bool) -> Option<Skipped<'a, T>> {
        if let Some(next) = node.next() {
            return Some(Skipped { node: next, first });
        }

        let (node, parent_id) = self.parents.pop()?;
        self.prev = Some(parent_id);
        Some(Skipped { node, first: false })
    }

    /// Advance the iteration.
    fn step(&mut self, node: Node<'a, T>, node_id: NonMax) -> Option<(Node<'a, T>, bool)> {
        if let Some(next) = node.first() {
            self.parents.push((node, node_id));
            return Some((next, true));
        }

        if let Some(next) = node.next() {
            self.prev = Some(node_id);
            return Some((next, false));
        }

        while let Some((parent, prev_id)) = self.parents.pop() {
            if let Some(next) = parent.next() {
                self.prev = Some(prev_id);
                return Some((next, false));
            }
        }

        None
    }
}
