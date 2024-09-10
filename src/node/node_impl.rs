use core::fmt;
use core::mem::size_of;
use core::ops::Range;

use crate::flavor::Flavor;
use crate::links::Links;
use crate::node::{Ancestors, Children, Event, Siblings, Walk, WalkEvents};
use crate::pointer::Pointer;
use crate::span::Span;

/// A node in the tree.
///
/// # Type parameters and bounds
///
/// The three type parameters of the tree determines the following properties:
/// * `T` is the data stored in the tree.
/// * `F` determines the [Flavor] of the tree, defining numerical bounds of
///   spans stored in the tree.
///
/// To use the default values, use the [Builder::new][crate::Builder::new]
/// constructor.
///
/// [Empty]: crate::Empty
pub struct Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    links: &'a Links<T, F::Index, F::Pointer>,
    tree: &'a [Links<T, F::Index, F::Pointer>],
}

impl<'a, T, F> Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    pub(crate) const fn new(
        links: &'a Links<T, F::Index, F::Pointer>,
        tree: &'a [Links<T, F::Index, F::Pointer>],
    ) -> Self {
        Self { links, tree }
    }

    /// Access the data associated with the node.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         ("number", 5),
    ///         ("ident", 3),
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(number.value(), "number");
    ///
    /// let ident = number.next().ok_or("missing ident")?;
    /// assert_eq!(ident.value(), "ident");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn value(&self) -> T {
        self.links.data.get()
    }

    /// Replace the value of the node with a new one, returning the old value.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         ("number", 5),
    ///         ("ident", 3),
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(number.value(), "number");
    /// assert_eq!(number.replace("other"), "number");
    /// assert_eq!(number.value(), "other");
    ///
    /// let ident = number.next().ok_or("missing ident")?;
    /// assert_eq!(ident.value(), "ident");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn replace(&self, value: T) -> T {
        self.links.data.replace(value)
    }

    /// Check if the current node has children or not.
    ///
    /// Nodes without children are also known as tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         ("number", 5),
    ///         ("ident", 3),
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert!(root.has_children());
    /// assert!(root.children().all(|n| !n.has_children()));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn has_children(&self) -> bool {
        self.links.first.is_some()
    }

    /// Get the span of the current node. The span of a node is the complete
    /// span of all its children.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.span(), Span::new(0, 8));
    ///
    /// let root2 = root.next().ok_or("missing second root")?;
    /// assert_eq!(root2.span(), Span::new(8, 13));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn span(&self) -> &Span<F::Index> {
        &self.links.span
    }

    /// Check if the current node is empty. In that it doesn't have any
    /// children.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::tree! {
    ///     "root",
    ///     "root2" => {
    ///         ("token2", 5)
    ///     }
    /// };
    ///
    /// let first = tree.first().ok_or("missing root")?;
    /// let last = first.next().ok_or("missing root2")?;
    ///
    /// assert!(first.is_empty());
    /// assert!(!last.is_empty());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.links.first.is_none()
    }

    /// Get the ancestors of this node.
    ///
    /// See [Ancestors] for documentation.
    #[must_use]
    pub fn ancestors(&self) -> Ancestors<'a, T, F> {
        Ancestors::new(Some(*self))
    }

    /// Get an iterator over the siblings of this node, including itself.
    ///
    /// See [Siblings] for documentation.
    #[must_use]
    pub fn siblings(&self) -> Siblings<'a, T, F> {
        Siblings::new(self.tree, self.links)
    }

    /// Get an iterator over the children of this node.
    ///
    /// See [Children] for documentation.
    #[must_use]
    pub fn children(&self) -> Children<'a, T, F> {
        Children::new(self.tree, self.links.first, self.links.last)
    }

    /// Walk the subtree forward starting with the first child of the current
    /// node.
    ///
    /// See [Walk] for documentation.
    #[must_use]
    pub fn walk(&self) -> Walk<'a, T, F> {
        Walk::new(self.tree, Some(self.id()), Event::Next)
    }

    /// Walk from the current node forwards and upwards through the tree.
    ///
    /// This does not include the current node in the walk.
    ///
    /// See [Walk] for documentation.
    #[must_use]
    pub fn walk_from(&self) -> Walk<'a, T, F> {
        Walk::new(self.tree, Some(self.id()), Event::Up)
    }

    /// Walk the node forwards in a depth-first fashion emitting events
    /// indicating how the rest of the tree is being traversed.
    ///
    /// See [`WalkEvents`] for documentation.
    #[must_use]
    pub fn walk_events(&self) -> WalkEvents<'a, T, F> {
        WalkEvents::new(self.tree, Some(self.id()), Event::Next)
    }
}

impl<'a, T, F> Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Get immediate parent to this node.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    /// assert!(root.parent().is_none());
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(number.value(), "number");
    ///
    /// let root = number.parent().ok_or("missing parent")?;
    /// assert_eq!(root.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn parent(&self) -> Option<Node<'a, T, F>> {
        self.node_at(self.links.parent?)
    }

    /// Get the previous sibling.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// let number = tree.first().and_then(|n| n.first()).ok_or("missing number")?;
    /// assert_eq!(number.value(), "number");
    /// assert!(number.prev().is_none());
    ///
    /// let ident = number.next().ok_or("missing ident")?;
    /// assert_eq!(ident.value(), "ident");
    ///
    /// let number = ident.prev().ok_or("missing number")?;
    /// assert_eq!(number.value(), "number");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn prev(&self) -> Option<Node<'a, T, F>> {
        self.node_at(self.links.prev?)
    }

    /// Get the next sibling.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing second root")?;
    /// assert_eq!(number.value(), "number");
    ///
    /// let ident = number.next().ok_or("missing second root")?;
    /// assert_eq!(ident.value(), "ident");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn next(&self) -> Option<Node<'a, T, F>> {
        self.node_at(self.links.next?)
    }

    /// Get the first child node.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(number.value(), "number");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn first(&self) -> Option<Node<'a, T, F>> {
        self.node_at(self.links.first?)
    }

    /// Get the last child node.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root2 = tree.last().ok_or("missing root2")?;
    /// assert_eq!(root2.value(), "root2");
    ///
    /// let whitespace = root2.last().ok_or("missing whitespace")?;
    /// assert_eq!(whitespace.value(), "whitespace");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn last(&self) -> Option<Node<'a, T, F>> {
        self.node_at(self.links.last?)
    }

    /// Find a preceeding node which matches the given predicate.
    ///
    /// A "preceeding node" is one which constitutes tokens the immediately
    /// preceedes the ones of the current node, so this function scans first the
    /// parents of the current node for a matching [`Node::prev`] sibling, and
    /// then traverses that matches [`Node::last`].
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child1" => {
    ///             ("token2", 1),
    ///             "child2" => {
    ///                 ("token1", 2)
    ///             }
    ///         },
    ///         "child3" => {
    ///             "child4" => {
    ///                 ("token1", 4),
    ///             }
    ///         }
    ///     }
    /// };
    ///
    /// let node = tree.node_with_range(3..3).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "child4");
    ///
    /// let found = node.find_preceding(|n| n.span().end == 3 && n.has_children());
    /// let found = found.expect("expected preceeding node");
    /// assert_eq!(found.value(), "child2");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn find_preceding<P>(&self, mut predicate: P) -> Option<Node<'a, T, F>>
    where
        P: FnMut(Node<'a, T, F>) -> bool,
    {
        // Step 1: Scan upwards until we find a previous s
        let mut n = *self;

        let mut n = loop {
            let Some(prev) = n.prev() else {
                n = n.parent()?;
                continue;
            };

            if predicate(prev) {
                break prev;
            }

            n = n.parent()?;
        };

        // Step 2: Scan last node while it matches the predicate.
        loop {
            let Some(last) = n.last() else {
                return Some(n);
            };

            if !predicate(last) {
                return Some(n);
            }

            n = last;
        }
    }

    fn node_at(&self, id: F::Pointer) -> Option<Node<'a, T, F>> {
        let cur = self.tree.get(id.get())?;

        Some(Self {
            links: cur,
            tree: self.tree,
        })
    }

    /// Get the identifier of the current node.
    ///
    /// Note that an id might be re-used across different trees. This behavior
    /// is never unsafe, but is not well-defined.
    ///
    /// This can be used to register a change in a [`ChangeSet`] later.
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    /// let root_id = tree.open("root")?;
    /// let child_id = tree.open("child")?;
    /// tree.close()?;
    ///
    /// let child2_id = tree.open("child2")?;
    /// tree.close()?;
    /// tree.close()?;
    ///
    /// let tree = tree.build()?;
    /// let root = tree.first().ok_or("missing root")?;
    /// let child = root.first().ok_or("missing child")?;
    /// let child2 = child.next().ok_or("missing child2")?;
    ///
    /// assert_eq!(root.id(), root_id);
    /// assert_eq!(child.id(), child_id);
    /// assert_eq!(child2.id(), child2_id);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// [`ChangeSet`]: crate::edit::ChangeSet
    #[must_use]
    pub fn id(&self) -> F::Pointer {
        // We're relying on the knowledge that the provided links reference is
        // inside of the tree of links.
        let current = self.links as *const _ as usize;
        let base = self.tree.as_ptr() as usize;
        let id = (current - base) / size_of::<Links<T, F::Index, F::Pointer>>();
        debug_assert!(id < self.tree.len(), "identifier outside of tree length");
        // SAFETY: It's impossible to construct a node with an offset which is
        // not a legal `NonMax`.
        unsafe { F::Pointer::new_unchecked(id) }
    }
}

impl<T, F> Node<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Access the [Span] of the node as a [Range].
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     },
    ///     "root2" => {
    ///         ("whitespace", 5)
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.range(), 0..8);
    ///
    /// let root2 = root.next().ok_or("missing second root")?;
    /// assert_eq!(root2.range(), 8..13);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.links.span.range()
    }
}

impl<T, F> fmt::Debug for Node<'_, T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.links.data.get())
            .field("span", &self.links.span)
            .finish()
    }
}

impl<T, F> Clone for Node<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F> Copy for Node<'_, T, F>
where
    T: Copy,
    F: Flavor,
{
}

impl<T, A, B> PartialEq<Node<'_, T, A>> for Node<'_, T, B>
where
    T: Copy + PartialEq,
    A: Flavor,
    B: Flavor<Index: PartialEq<A::Index>>,
{
    fn eq(&self, other: &Node<'_, T, A>) -> bool {
        self.links.data.get() == other.links.data.get() && self.links.span == other.links.span
    }
}

impl<T, F> Eq for Node<'_, T, F>
where
    T: Copy + Eq,
    F: Flavor<Index: Eq>,
{
}
