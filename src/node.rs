use std::fmt;
use std::mem::size_of;
use std::ops::Range;

use crate::links::Links;
use crate::non_max::NonMax;
use crate::tree::Kind;
use crate::{Ancestors, Children, Id, Siblings, Span, Walk, WalkEvents};

/// A node in the tree.
pub struct Node<'a, T> {
    links: &'a Links<T>,
    tree: &'a [Links<T>],
}

impl<'a, T> Node<'a, T> {
    pub(crate) const fn new(links: &'a Links<T>, tree: &'a [Links<T>]) -> Self {
        Self { links, tree }
    }

    /// Get the identifier of the current node.
    ///
    /// This can be used to register a change in a [ChangeSet][crate::ChangeSet]
    /// later.
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = TreeBuilder::new();
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
    /// # Ok(()) }
    /// ```
    pub fn id(&self) -> Id {
        let current = self.links as *const _ as usize;
        let base = self.tree.as_ptr() as usize;
        let id = (current - base) / size_of::<Links<T>>();
        // SAFETY: It's impossible to construct a node with an offset which is
        // not a legal `NonMax`.
        unsafe { Id::new(NonMax::new_unchecked(id)) }
    }

    /// Access the data associated with the node.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         ("number", 5),
    ///         ("ident", 3),
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(*number.value(), "number");
    ///
    /// let ident = number.next().ok_or("missing ident")?;
    /// assert_eq!(*ident.value(), "ident");
    /// # Ok(()) }
    /// ```
    pub fn value(&self) -> &'a T {
        &self.links.data
    }

    /// Access the kind of the node.
    ///
    /// Terminating nodes are [Kind::Token] and intermediary nodes are
    /// [Kind::Node].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Kind;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         ("number", 5),
    ///         ("ident", 3),
    ///     }
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.kind(), Kind::Node);
    ///
    /// assert!(root.children().all(|n| matches!(n.kind(), Kind::Token)));
    /// # Ok(()) }
    /// ```
    pub const fn kind(&self) -> Kind {
        self.links.kind
    }

    /// Get the span of the current node. The span of a node is the complete
    /// span of all its children.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// # Ok(()) }
    /// ```
    pub const fn span(&self) -> Span {
        self.links.span
    }

    /// Access the [Span] of the node as a [Range].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// # Ok(()) }
    /// ```
    pub const fn range(&self) -> Range<usize> {
        self.links.span.range()
    }

    /// Check if the current node is empty. In that it doesn't have any
    /// children.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tree = syntree::tree! {
    ///     "root",
    ///     "root2" => {
    ///         ("token", 5)
    ///     }
    /// };
    ///
    /// let first = tree.first().ok_or("missing first root node")?;
    /// let last = first.next().ok_or("missing last root node")?;
    ///
    /// assert!(first.is_empty());
    /// assert!(!last.is_empty());
    /// # Ok(()) }
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.links.first.is_none()
    }

    /// Get the ancestors of this node.
    ///
    /// See [Ancestors] for documentation.
    pub fn ancestors(&self) -> Ancestors<'a, T> {
        Ancestors::new(Some(*self))
    }

    /// Get an iterator over the siblings of this node, including itself.
    ///
    /// See [Siblings] for documentation.
    pub fn siblings(&self) -> Siblings<'a, T> {
        Siblings::new(self.tree, self.links)
    }

    /// Get an iterator over the children of this node.
    ///
    /// See [Children] for documentation.
    pub fn children(&self) -> Children<'a, T> {
        Children::new(self.tree, self.links.first, self.links.last)
    }

    /// Walk the subtree forward starting with the first child of the current
    /// node.
    ///
    /// See [Walk] for documentation.
    pub fn walk(&self) -> Walk<'a, T> {
        Walk::new(self.tree, self.links.first)
    }

    /// Walk the node forwards in a depth-first fashion emitting events
    /// indicating how the rest of the tree is being traversed.
    ///
    /// See [WalkEvents] for documentation.
    pub fn walk_events(&self) -> WalkEvents<'a, T> {
        WalkEvents::new(self.tree, self.links.first)
    }

    /// Get immediate parent to this node.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// assert_eq!(*root.value(), "root");
    /// assert!(root.parent().is_none());
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(*number.value(), "number");
    ///
    /// let root = number.parent().ok_or("missing parent")?;
    /// assert_eq!(*root.value(), "root");
    /// # Ok(()) }
    /// ```
    pub fn parent(&self) -> Option<Node<'a, T>> {
        self.node_at(self.links.parent?)
    }

    /// Get the previous sibling.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// assert_eq!(*number.value(), "number");
    /// assert!(number.prev().is_none());
    ///
    /// let ident = number.next().ok_or("missing ident")?;
    /// assert_eq!(*ident.value(), "ident");
    ///
    /// let number = ident.prev().ok_or("missing number")?;
    /// assert_eq!(*number.value(), "number");
    /// # Ok(()) }
    /// ```
    pub fn prev(self) -> Option<Node<'a, T>> {
        self.node_at(self.links.prev?)
    }

    /// Get the next sibling.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// assert_eq!(*root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing second root")?;
    /// assert_eq!(*number.value(), "number");
    ///
    /// let ident = number.next().ok_or("missing second root")?;
    /// assert_eq!(*ident.value(), "ident");
    /// # Ok(()) }
    /// ```
    pub fn next(self) -> Option<Node<'a, T>> {
        self.node_at(self.links.next?)
    }

    /// Get the first child node.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// assert_eq!(*root.value(), "root");
    ///
    /// let number = root.first().ok_or("missing number")?;
    /// assert_eq!(*number.value(), "number");
    /// # Ok(()) }
    /// ```
    pub fn first(&self) -> Option<Node<'a, T>> {
        self.node_at(self.links.first?)
    }

    /// Get the last child node.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// assert_eq!(*root2.value(), "root2");
    ///
    /// let whitespace = root2.last().ok_or("missing whitespace")?;
    /// assert_eq!(*whitespace.value(), "whitespace");
    /// # Ok(()) }
    /// ```
    pub fn last(&self) -> Option<Node<'a, T>> {
        self.node_at(self.links.last?)
    }

    fn node_at(&self, id: NonMax) -> Option<Node<'a, T>> {
        let cur = self.tree.get(id.get())?;

        Some(Self {
            links: cur,
            tree: self.tree,
        })
    }
}

impl<'a, T> fmt::Debug for Node<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.links.data)
            .field("kind", &self.links.kind)
            .field("span", &self.links.span)
            .finish()
    }
}

impl<'a, T> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Node<'a, T> {}

impl<'a, T> PartialEq for Node<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.links.data == other.links.data
            && self.links.kind == other.links.kind
            && self.links.span == other.links.span
    }
}

impl<'a, T> Eq for Node<'a, T> where T: Eq {}
