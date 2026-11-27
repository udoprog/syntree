use core::fmt;
use core::ops::Range;

use crate::flavor::Flavor;
use crate::links::NodeLinks;
use crate::node::{
    Ancestors, Children, Element, Event, Id, NodeOrToken, Siblings, Walk, WalkEvents,
};
use crate::pointer::Pointer;
use crate::span::Span;
use crate::tree::Tree;

/// An interior node in the tree, one which was opened with
/// [`Builder::open`][crate::Builder::open] or wrapped with
/// [`Builder::close_at`][crate::Builder::close_at].
///
/// Tokens are [`Token`][crate::node::Token]s, and an iterator which mixes both
/// yields [`Element`]s. An [`Element`] has most of the methods of a node, so a
/// child can usually be used without first checking what it is.
///
/// # Type parameters and bounds
///
/// The two type parameters of the tree determine the following properties:
/// * `T` is the data stored in the tree.
/// * `F` determines the [Flavor] of the tree, defining numerical bounds of
///   spans and node identifiers stored in the tree.
///
/// A tree built through [Builder::new][crate::Builder::new] uses
/// [`FlavorDefault`][crate::FlavorDefault].
pub struct Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    tree: &'a Tree<T, F>,
    links: &'a NodeLinks<T, F::Index, F::Pointer>,
    id: F::Pointer,
}

impl<'a, T, F> Node<'a, T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a handle to the node with the given id.
    pub(crate) fn new(tree: &'a Tree<T, F>, id: F::Pointer) -> Option<Self> {
        let links = tree.node_links(id)?;
        Some(Self { tree, links, id })
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
    /// let root = tree.first().and_then(|e| e.into_node()).ok_or("missing root")?;
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
    ///         "number" => {},
    ///         ("ident", 3),
    ///     }
    /// };
    ///
    /// let root = tree.first().and_then(|e| e.into_node()).ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    ///
    /// let number = root.first().and_then(|e| e.into_node()).ok_or("missing number")?;
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

    /// Get the identifier of the node.
    ///
    /// Node ids index the interior nodes of the tree and are separate from
    /// [token ids][crate::Token::id], so a node and a token can share an id.
    /// Use [`Tree::get`] to get the node back. An id might be re-used across
    /// different trees. This behavior is never unsafe, but is not
    /// well-defined.
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
    /// let root = tree.first().and_then(|e| e.into_node()).ok_or("missing root")?;
    /// let child = root.first().and_then(|e| e.into_node()).ok_or("missing child")?;
    /// let child2 = child.next().and_then(|e| e.into_node()).ok_or("missing child2")?;
    ///
    /// assert_eq!(root.id(), root_id);
    /// assert_eq!(child.id(), child_id);
    /// assert_eq!(child2.id(), child2_id);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// [`Tree::get`]: crate::Tree::get
    /// [`ChangeSet`]: crate::edit::ChangeSet
    #[must_use]
    pub fn id(&self) -> F::Pointer {
        self.id
    }

    /// Get the span of the node. The span of a node is the complete span of
    /// all its children.
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
    /// let root = tree.first().and_then(|e| e.into_node()).ok_or("missing root")?;
    /// assert_eq!(*root.span(), Span::new(0, 8));
    ///
    /// let root2 = root.next().ok_or("missing second root")?;
    /// assert_eq!(root2.span(), Span::new(8, 13));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn span(&self) -> &Span<F::Index> {
        &self.links.span
    }

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

    /// Check if the node has children, tokens or nodes.
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
    pub fn has_children(&self) -> bool {
        self.first().is_some()
    }

    /// Check if the node is empty, in that it has no children.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree = syntree::tree! {
    ///     "root" => {},
    ///     "root2" => {
    ///         ("token2", 5)
    ///     }
    /// };
    ///
    /// let first = tree.first().and_then(|e| e.into_node()).ok_or("missing root")?;
    /// let last = tree.last().and_then(|e| e.into_node()).ok_or("missing root2")?;
    ///
    /// assert!(first.is_empty());
    /// assert!(!last.is_empty());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.has_children()
    }

    /// Get the parent node, `None` at the root of the tree.
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
        Node::new(self.tree, self.links.parent?)
    }

    /// Get the previous sibling, a node or a token.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         ("lit", 5),
    ///         "ident" => {
    ///             ("lit", 3)
    ///         }
    ///     }
    /// };
    ///
    /// let ident = tree.first().and_then(|n| n.last()).ok_or("missing ident")?;
    /// assert_eq!(ident.value(), "ident");
    ///
    /// let lit = ident.prev().ok_or("missing lit")?;
    /// assert_eq!(lit.value(), "lit");
    /// assert!(lit.is_token());
    /// assert!(lit.prev().is_none());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn prev(&self) -> Option<Element<'a, T, F>> {
        let id = self.tree.prev_at(
            self.links.parent,
            self.links.tok_start.get(),
            self.links.prev,
        )?;
        self.tree.element(id)
    }

    /// Get the next sibling, a node or a token.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "number" => {
    ///             ("lit", 5)
    ///         },
    ///         ("ident", 3)
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
    /// assert!(ident.is_token());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn next(&self) -> Option<Element<'a, T, F>> {
        let id = self
            .tree
            .next_at(self.links.parent, self.links.tok_end.get(), self.links.next)?;
        self.tree.element(id)
    }

    /// Get the first child, a node or a token.
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
    pub fn first(&self) -> Option<Element<'a, T, F>> {
        let id = self
            .tree
            .next_at(Some(self.id), self.links.tok_start.get(), self.links.first)?;
        self.tree.element(id)
    }

    /// Get the last child, a node or a token.
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
    pub fn last(&self) -> Option<Element<'a, T, F>> {
        let id = self
            .tree
            .prev_at(Some(self.id), self.links.tok_end.get(), self.links.last)?;
        self.tree.element(id)
    }

    /// Get an iterator over this node and its ancestors, starting with the
    /// node itself and ending with its root.
    ///
    /// See [Ancestors] for documentation.
    #[must_use]
    pub fn ancestors(&self) -> Ancestors<'a, T, F> {
        Ancestors::new(Some(NodeOrToken::Node(*self)))
    }

    /// Get an iterator over this node and its following siblings, nodes and
    /// tokens.
    ///
    /// Previous siblings are not included.
    ///
    /// See [Siblings] for documentation.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "a",
    ///         "b" => {},
    ///         "c",
    ///         "d",
    ///     }
    /// };
    ///
    /// let b = tree.first().and_then(|n| n.first()).and_then(|n| n.next()).ok_or("missing b")?;
    /// assert_eq!(b.value(), "b");
    ///
    /// assert_eq!(
    ///     b.siblings().map(|n| n.value()).collect::<Vec<_>>(),
    ///     ["b", "c", "d"]
    /// );
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn siblings(&self) -> Siblings<'a, T, F> {
        Siblings::new(self.tree, self.links.parent, Id::Node(self.id), None)
    }

    /// Iterate over the children of the node, nodes and tokens in order.
    ///
    /// See [Children] for documentation.
    #[must_use]
    pub fn children(&self) -> Children<'a, T, F> {
        Children::new(
            self.tree,
            Some(self.id),
            (self.links.tok_start.get(), self.links.first),
            (self.links.tok_end.get(), self.links.last),
        )
    }

    /// Walk the tree forward in depth-first order starting with the current
    /// node.
    ///
    /// The walk does not stop at the end of this node's subtree, it continues
    /// with the following siblings of this node and of its ancestors. Use
    /// [`Walk::inside`] to only visit this node and its descendants.
    ///
    /// See [Walk] for documentation.
    #[must_use]
    pub fn walk(&self) -> Walk<'a, T, F> {
        Walk::new(self.tree, Some(Id::Node(self.id)), None, Event::Next)
    }

    /// Walk from the current node forwards and upwards through the tree.
    ///
    /// This does not include the current node or its descendants in the walk.
    ///
    /// See [Walk] for documentation.
    #[must_use]
    pub fn walk_from(&self) -> Walk<'a, T, F> {
        Walk::new(self.tree, Some(Id::Node(self.id)), None, Event::Up)
    }

    /// Walk the node forwards in a depth-first fashion emitting events
    /// indicating how the rest of the tree is being traversed.
    ///
    /// See [`WalkEvents`] for documentation.
    #[must_use]
    pub fn walk_events(&self) -> WalkEvents<'a, T, F> {
        WalkEvents::new(self.tree, Some(Id::Node(self.id)), None, Event::Next)
    }

    /// Find a preceding element which matches the given predicate.
    ///
    /// A "preceding element" is one whose tokens immediately precede the
    /// tokens of the current node. This is found in two steps:
    ///
    /// 1. Starting with the current node and moving through its ancestors, find
    ///    the first [`prev`][Node::prev] sibling which matches the predicate.
    /// 2. From that sibling, descend through [`last`][Node::last] children for
    ///    as long as each matches the predicate, and return the last match.
    ///
    /// Step 2 stops at the first last child which does not match, so a
    /// matching element nested below a non-matching last child is not found.
    ///
    /// See [`Element::find_preceding`] for an example.
    pub fn find_preceding<P>(&self, predicate: P) -> Option<Element<'a, T, F>>
    where
        P: FnMut(Element<'a, T, F>) -> bool,
    {
        NodeOrToken::Node(*self).find_preceding(predicate)
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

/// Nodes compare equal when their [values][Node::value] and
/// [spans][Node::span] are equal, regardless of their position in a tree.
///
/// Compare [`Node::id`] to check whether two nodes are the same node.
///
/// # Examples
///
/// ```
/// let tree = syntree::tree! {
///     "x" => {
///         "x" => {}
///     }
/// };
///
/// let root = tree.first().and_then(|e| e.into_node()).ok_or("missing root")?;
/// let child = root.first().and_then(|e| e.into_node()).ok_or("missing child")?;
///
/// assert_eq!(root, child);
/// assert_ne!(root.id(), child.id());
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
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
