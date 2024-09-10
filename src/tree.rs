use core::fmt;
use core::ops::{Deref, Range};

use alloc::vec::Vec;

use crate::flavor::Flavor;
use crate::index::{BinarySearch, Index, Indexes};
use crate::links::Links;
use crate::node::{Children, Walk, WalkEvents};
use crate::node::{Event, Node};
use crate::pointer::{Pointer, Width};
use crate::span::Span;

/// A syntax tree.
///
/// A tree is constructed through a [Builder][crate::Builder] or by modifying an
/// existing tree through a [ChangeSet][crate::edit::ChangeSet].
///
/// # Type parameters and bounds
///
/// The three type parameters of the tree determines the following properties:
/// * `T` is the data stored in the tree.
/// * `I` determines the numerical bounds of spans stored in the tree through
///   the [Index] trait, if set to [Empty][crate::Empty] the tree does not store
///   any spans.
/// * `W` determines the bounds of pointers in the tree through the [Width]
///   trait, this decides how many elements that can be stored in the tree.
///
/// To use the default values, use the [Builder::new][crate::Builder::new]
/// constructor.
pub struct Tree<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Links in the tree.
    tree: Vec<Links<T, F::Index, F::Pointer>>,
    /// The span of the whole tree.
    span: Span<F::Index>,
    /// Token indexes for range searches. This contains the value of the token
    /// cursor each time it is modified and allow for binary searching for
    /// sequences of nodes which corresponds to the given index.
    indexes: F::Indexes,
    /// The first node in the tree.
    first: Option<F::Pointer>,
    /// The last node in the tree.
    last: Option<F::Pointer>,
}

impl<T, F> Tree<T, F>
where
    T: Copy,
    F: Flavor,
{
    /// Construct a new empty tree.
    pub(crate) const fn new_with() -> Self {
        Self {
            tree: Vec::new(),
            span: Span::point(F::Index::EMPTY),
            indexes: F::Indexes::EMPTY,
            first: None,
            last: None,
        }
    }

    /// Construct a new tree with the given capacity.
    #[cfg(feature = "std")]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            tree: Vec::with_capacity(capacity),
            span: Span::point(F::Index::EMPTY),
            indexes: F::Indexes::EMPTY,
            first: None,
            last: None,
        }
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
    /// assert_eq!(tree.span(), Span::new(0, 13));
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub const fn span(&self) -> &Span<F::Index> {
        &self.span
    }

    /// Get mutable span from the tree.
    pub(crate) fn span_mut(&mut self) -> &mut Span<F::Index> {
        &mut self.span
    }

    /// The total number of elements in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree: syntree::Builder<()> = syntree::Builder::new();
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.len(), 0);
    ///
    /// let mut tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("token", 2)
    ///         },
    ///         ("whitespace", 1),
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert_eq!(tree.len(), 5);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the current tree is empty. In that it doesn't have any
    /// childrens at the root of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree: syntree::Builder<()> = syntree::Builder::new();
    /// let tree = tree.build()?;
    /// assert!(tree.is_empty());
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Get the capacity of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tree: syntree::Builder<()> = syntree::Builder::new();
    /// let tree = tree.build()?;
    ///
    /// assert_eq!(tree.capacity(), 0);
    ///
    /// let mut tree = syntree::tree! {
    ///     "root" => {
    ///         "child" => {
    ///             ("token", 2)
    ///         },
    ///         ("whitespace", 1),
    ///         "child2" => {}
    ///     }
    /// };
    ///
    /// assert!(tree.capacity() >= 5);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn capacity(&self) -> usize {
        self.tree.capacity()
    }

    /// Get all root nodes in the tree.
    ///
    /// See [Children] for documentation.
    pub fn children(&self) -> Children<'_, T, F> {
        Children::new(&self.tree, self.first, self.last)
    }

    /// Walk the tree forwards in a depth-first fashion visiting every node
    /// once.
    ///
    /// See [`Walk`] for documentation.
    pub fn walk(&self) -> Walk<'_, T, F> {
        Walk::new(self.tree.as_slice(), self.first, Event::Next)
    }

    /// Walk the tree forwards in a depth-first fashion emitting events
    /// indicating how the tree is being traversed.
    ///
    /// See [`WalkEvents`] for documentation.
    pub fn walk_events(&self) -> WalkEvents<'_, T, F> {
        WalkEvents::new(self.tree.as_slice(), self.first, Event::Next)
    }

    /// Get the first child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {},
    ///     "root2" => {}
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn first(&self) -> Option<Node<'_, T, F>> {
        self.get(self.first?)
    }

    /// Get the last child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {},
    ///     "root2" => {}
    /// };
    ///
    /// let root = tree.last().ok_or("missing root")?;
    /// assert_eq!(root.value(), "root2");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn last(&self) -> Option<Node<'_, T, F>> {
        self.get(self.last?)
    }

    /// Get the tree links mutably.
    pub(crate) fn links_mut(&mut self) -> (&mut Option<F::Pointer>, &mut Option<F::Pointer>) {
        (&mut self.first, &mut self.last)
    }

    /// Get a mutable reference to an element in the tree.
    pub(crate) fn get_mut(
        &mut self,
        id: F::Pointer,
    ) -> Option<&mut Links<T, F::Index, F::Pointer>> {
        self.tree.get_mut(id.get())
    }

    /// Push a new node into the tree with the specified links.
    pub(crate) fn push(&mut self, links: Links<T, F::Index, F::Pointer>) {
        self.tree.push(links);
    }

    /// Push the given index.
    pub(crate) fn indexes_mut(&mut self) -> &mut F::Indexes {
        &mut self.indexes
    }

    /// Optionally get the links at the given location.
    pub(crate) fn links_at_mut(
        &mut self,
        index: F::Pointer,
    ) -> Option<&mut Links<T, F::Index, F::Pointer>> {
        self.tree.get_mut(index.get())
    }

    /// Get the ndoe at the given index.
    ///
    /// Note that an id might be re-used across different trees. This behavior
    /// is never unsafe, but is not well-defined.
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
    /// let node = tree.first().and_then(|n| n.last()).ok_or("missing ident")?;
    /// assert_eq!(node.value(), "ident");
    ///
    /// let id = node.id();
    /// let node = tree.get(id).ok_or("missing ident")?;
    /// assert_eq!(node.value(), "ident");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn get(&self, id: F::Pointer) -> Option<Node<'_, T, F>> {
        let cur = self.tree.get(id.get())?;
        Some(Node::new(cur, &self.tree))
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
    /// assert_eq!(tree.range(), 0..13);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.span.range()
    }

    /// Query for the node that matches the given range.
    ///
    /// This query finds the node which contains the entirety of the given
    /// [Range].
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child1" => {
    ///             ("token1", 3)
    ///         },
    ///         "child2" => {
    ///             "nested1" => {
    ///                 ("token1", 4),
    ///             },
    ///             ("token4", 1),
    ///         },
    ///         "child3" => {
    ///             ("token5", 5)
    ///         }
    ///     },
    ///     "root2" => {}
    /// };
    ///
    /// let node = tree.node_with_range(0..0).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "child1");
    ///
    /// let node = tree.node_with_range(0..3).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "child1");
    ///
    /// let node = tree.node_with_range(3..3).ok_or("missing 3")?;
    /// assert_eq!(node.value(), "nested1");
    ///
    /// let node = tree.node_with_range(3..7).ok_or("missing 3..7")?;
    /// assert_eq!(node.value(), "nested1");
    ///
    /// let node = tree.node_with_range(7..7).ok_or("missing 7")?;
    /// assert_eq!(node.value(), "child2");
    ///
    /// let node = tree.node_with_range(7..8).ok_or("missing 7..8")?;
    /// assert_eq!(node.value(), "child2");
    ///
    /// let node = tree.node_with_range(8..8).ok_or("missing 8")?;
    /// assert_eq!(node.value(), "child3");
    ///
    /// let node = tree.node_with_range(8..13).ok_or("missing 9")?;
    /// assert_eq!(node.value(), "child3");
    ///
    /// let node = tree.node_with_range(2..4).ok_or("missing 2..4")?;
    /// assert_eq!(node.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// Range queries work as expected with checkpoints:
    ///
    /// ```
    /// let mut tree = syntree::Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    /// tree.open("child")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    /// tree.close_at(&c, "root")?;
    /// tree.token("sibling", 3)?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let child = tree.node_with_range(0..3).ok_or("missing at 0..3")?;
    /// assert_eq!(child.value(), "child");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn node_with_range(&self, span: Range<usize>) -> Option<Node<'_, T, F>>
    where
        F::Indexes: Deref<Target: BinarySearch<F::Index>>,
    {
        let start = F::Index::from_usize(span.start)?;
        let end = F::Index::from_usize(span.end)?;
        self.node_with_span_internal(start, end)
    }

    /// Query the tree for the first node which encapsulates the whole `span`.
    ///
    /// This query finds the node which contains the entirety of the given
    /// [Span].
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::Span;
    ///
    /// let tree = syntree::tree! {
    ///     "root" => {
    ///         "child1" => {
    ///             ("token1", 3)
    ///         },
    ///         "child2" => {
    ///             "nested1" => {
    ///                 ("token1", 4),
    ///             },
    ///             ("token4", 1),
    ///         },
    ///         "child3" => {
    ///             ("token5", 5)
    ///         }
    ///     },
    ///     "root2" => {}
    /// };
    ///
    /// let node = tree.node_with_span(Span::point(0)).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "child1");
    ///
    /// let node = tree.node_with_span(Span::new(0, 3)).ok_or("missing 0")?;
    /// assert_eq!(node.value(), "child1");
    ///
    /// let node = tree.node_with_span(Span::point(3)).ok_or("missing 3")?;
    /// assert_eq!(node.value(), "nested1");
    ///
    /// let node = tree.node_with_span(Span::new(3, 7)).ok_or("missing 3..7")?;
    /// assert_eq!(node.value(), "nested1");
    ///
    /// let node = tree.node_with_span(Span::point(7)).ok_or("missing 7")?;
    /// assert_eq!(node.value(), "child2");
    ///
    /// let node = tree.node_with_span(Span::new(7, 8)).ok_or("missing 7..8")?;
    /// assert_eq!(node.value(), "child2");
    ///
    /// let node = tree.node_with_span(Span::point(8)).ok_or("missing 8")?;
    /// assert_eq!(node.value(), "child3");
    ///
    /// let node = tree.node_with_span(Span::new(8, 13)).ok_or("missing 9")?;
    /// assert_eq!(node.value(), "child3");
    ///
    /// let node = tree.node_with_span(Span::new(2, 4)).ok_or("missing 2..4")?;
    /// assert_eq!(node.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// Range queries work as expected with checkpoints:
    ///
    /// ```
    /// use syntree::{Span, Builder};
    ///
    /// let mut tree = Builder::new();
    ///
    /// let c = tree.checkpoint()?;
    ///
    /// tree.open("child1")?;
    /// tree.token("lit", 3)?;
    /// tree.close()?;
    ///
    /// tree.open("child2")?;
    /// tree.token("lit", 2)?;
    /// tree.close()?;
    ///
    /// tree.close_at(&c, "root")?;
    ///
    /// let tree = tree.build()?;
    ///
    /// let child = tree.node_with_span(Span::point(0)).ok_or("missing at point 5")?;
    /// assert_eq!(child.value(), "child1");
    ///
    /// let child = tree.node_with_span(Span::new(0, 3)).ok_or("missing at 0..3")?;
    /// assert_eq!(child.value(), "child1");
    ///
    /// let child = tree.node_with_span(Span::new(3, 5)).ok_or("missing at 3..5")?;
    /// assert_eq!(child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::new(4, 5)).ok_or("missing at 4..5")?;
    /// assert_eq!(child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::new(3, 4)).ok_or("missing at 3..4")?;
    /// assert_eq!(child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::point(3)).ok_or("missing at point 5")?;
    /// assert_eq!(child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::new(2, 5)).ok_or("missing at 2..5")?;
    /// assert_eq!(child.value(), "root");
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn node_with_span(&self, span: Span<F::Index>) -> Option<Node<'_, T, F>>
    where
        F::Indexes: Deref<Target: BinarySearch<F::Index>>,
    {
        self.node_with_span_internal(span.start, span.end)
    }

    fn node_with_span_internal(&self, start: F::Index, end: F::Index) -> Option<Node<'_, T, F>>
    where
        F::Indexes: Deref<Target: BinarySearch<F::Index>>,
    {
        let result = self.indexes.binary_search(start);

        let n = match result {
            Ok(n) => n.saturating_add(1),
            Err(n) => n,
        };

        let mut node = self.get(*self.indexes.get(n)?)?;

        while let Some(parent) = node.parent() {
            node = parent;

            if parent.span().end >= end {
                break;
            }
        }

        Some(node)
    }
}

impl<T, F> Clone for Tree<T, F>
where
    T: Copy,
    F: Flavor<Indexes: Clone, Width: Width<Pointer: Clone>>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            span: self.span,
            indexes: self.indexes.clone(),
            first: self.first,
            last: self.last,
        }
    }
}

impl<T, F> Default for Tree<T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self::new_with()
    }
}

impl<T, A, B> PartialEq<Tree<T, A>> for Tree<T, B>
where
    T: Copy + PartialEq,
    A: Flavor,
    B: Flavor<Index: PartialEq<A::Index>>,
{
    fn eq(&self, other: &Tree<T, A>) -> bool {
        struct Item<'a, T, F>((isize, Node<'a, T, F>))
        where
            T: Copy,
            F: Flavor;

        // NB: this is needed because the constraints on tuples doesn't allow
        // for `A` and `B` to be different.
        impl<'a, T, A, B> PartialEq<Item<'a, T, A>> for Item<'a, T, B>
        where
            T: Copy + PartialEq,
            A: Flavor,
            B: Flavor<Index: PartialEq<A::Index>>,
        {
            #[inline]
            fn eq(&self, other: &Item<'a, T, A>) -> bool {
                self.0 .0 == other.0 .0 && self.0 .1.eq(&other.0 .1)
            }
        }

        self.walk()
            .with_depths()
            .map(Item)
            .eq(other.walk().with_depths().map(Item))
    }
}

impl<T, F> Eq for Tree<T, F>
where
    T: Copy + Eq,
    F: Flavor<Index: Eq>,
{
}

impl<T, F> fmt::Debug for Tree<T, F>
where
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct List<'a, T, F>(&'a Tree<T, F>)
        where
            T: Copy,
            F: Flavor;

        impl<T, F> fmt::Debug for List<'_, T, F>
        where
            T: Copy + fmt::Debug,
            F: Flavor<Index: fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.walk().with_depths()).finish()
            }
        }

        f.debug_tuple("Tree").field(&List(self)).finish()
    }
}
