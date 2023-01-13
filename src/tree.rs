use core::fmt;
use core::ops::Range;

use crate::links::Links;
use crate::non_max::NonMax;
use crate::span::{self, usize_to_index, Index};
use crate::{Children, Node, Span, Walk, WalkEvents};

/// The kind of a node in the [Tree].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// A node.
    Node,
    /// The token and a corresponding span.
    Token,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TreeIndex {
    pub(crate) index: Index,
    pub(crate) id: NonMax,
}

/// A syntax tree.
#[derive(Clone)]
pub struct Tree<T, S = Span> {
    /// Links in the tree.
    tree: Vec<Links<T, S>>,
    /// The span of the whole tree.
    span: S,
    /// Token indexes for range searches. This contains the value of the token
    /// cursor each time it is modified and allow for binary searching for
    /// sequences of nodes which corresponds to the given index.
    indexes: Vec<TreeIndex>,
    /// The first element in the tree.
    first: Option<NonMax>,
    /// The last element in the tree.
    last: Option<NonMax>,
}

impl<T, S> Tree<T, S> {
    /// Construct a new empty tree.
    pub(crate) const fn new_with() -> Self
    where
        S: span::Builder,
    {
        Self {
            tree: Vec::new(),
            span: S::EMPTY,
            indexes: Vec::new(),
            first: None,
            last: None,
        }
    }

    /// Construct a new tree with the given capacity.
    pub(crate) fn with_capacity(capacity: usize) -> Self
    where
        S: span::Builder,
    {
        Self {
            tree: Vec::with_capacity(capacity),
            span: S::EMPTY,
            indexes: Vec::new(),
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub const fn span(&self) -> &S {
        &self.span
    }

    /// Get mutable span from the tree.
    pub(crate) fn span_mut(&mut self) -> &mut S {
        &mut self.span
    }

    /// The total number of elements in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// let mut tree = TreeBuilder::<()>::new();
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
    ///         "child2"
    ///     }
    /// };
    ///
    /// assert_eq!(tree.len(), 5);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
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
    /// use syntree::TreeBuilder;
    ///
    /// let mut tree = TreeBuilder::<()>::new();
    /// let tree = tree.build()?;
    /// assert!(tree.is_empty());
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Get the capacity of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// let mut tree = TreeBuilder::<()>::new();
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
    ///         "child2"
    ///     }
    /// };
    ///
    /// assert!(tree.capacity() >= 5);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn capacity(&self) -> usize {
        self.tree.capacity()
    }

    /// Get all root nodes in the tree.
    ///
    /// See [Children] for documentation.
    pub fn children(&self) -> Children<'_, T, S> {
        Children::new(&self.tree, self.first, self.last)
    }

    /// Walk the tree forwards in a depth-first fashion visiting every node once.
    ///
    /// See [`Walk`] for documentation.
    pub fn walk(&self) -> Walk<'_, T, S> {
        Walk::new(self.tree.as_slice(), self.first)
    }

    /// Walk the tree forwards in a depth-first fashion emitting events
    /// indicating how the tree is being traversed.
    ///
    /// See [`WalkEvents`] for documentation.
    pub fn walk_events(&self) -> WalkEvents<'_, T, S> {
        WalkEvents::new(self.tree.as_slice(), self.first)
    }

    /// Get the first child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root",
    ///     "root2"
    /// };
    ///
    /// let root = tree.first().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn first(&self) -> Option<Node<'_, T, S>> {
        self.node_at(self.first?)
    }

    /// Get the last child node in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     "root",
    ///     "root2"
    /// };
    ///
    /// let root = tree.last().ok_or("missing root")?;
    /// assert_eq!(*root.value(), "root2");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn last(&self) -> Option<Node<'_, T, S>> {
        self.node_at(self.last?)
    }

    /// The first id currently being set.
    pub(crate) fn first_id(&self) -> Option<NonMax> {
        self.first
    }

    /// Get the tree links mutably.
    pub(crate) fn links_mut(&mut self) -> (&mut Option<NonMax>, &mut Option<NonMax>) {
        (&mut self.first, &mut self.last)
    }

    /// Get a mutable reference to an element in the tree.
    pub(crate) fn get_mut(&mut self, id: NonMax) -> Option<&mut Links<T, S>> {
        self.tree.get_mut(id.get())
    }

    /// Push a new element onto the tree.
    pub(crate) fn push(&mut self, links: Links<T, S>) {
        self.tree.push(links);
    }

    /// Push the given index.
    pub(crate) fn push_index(&mut self, index: Index, id: NonMax) {
        self.indexes.push(TreeIndex { index, id });
    }

    /// Optionally get the links at the given location.
    pub(crate) fn links_at_mut(&mut self, index: NonMax) -> Option<&mut Links<T, S>> {
        self.tree.get_mut(index.get())
    }

    /// Construct a node at the given location.
    pub(crate) fn node_at(&self, index: NonMax) -> Option<Node<'_, T, S>> {
        let cur = self.tree.get(index.get())?;
        Some(Node::new(cur, &self.tree))
    }
}

impl<T> Tree<T, Span> {
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
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn range(&self) -> Range<usize> {
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
    ///     "root2"
    /// };
    ///
    /// let node = tree.node_with_range(0..0).ok_or("missing 0")?;
    /// assert_eq!(*node.value(), "child1");
    ///
    /// let node = tree.node_with_range(0..3).ok_or("missing 0")?;
    /// assert_eq!(*node.value(), "child1");
    ///
    /// let node = tree.node_with_range(3..3).ok_or("missing 3")?;
    /// assert_eq!(*node.value(), "nested1");
    ///
    /// let node = tree.node_with_range(3..7).ok_or("missing 3..7")?;
    /// assert_eq!(*node.value(), "nested1");
    ///
    /// let node = tree.node_with_range(7..7).ok_or("missing 7")?;
    /// assert_eq!(*node.value(), "child2");
    ///
    /// let node = tree.node_with_range(7..8).ok_or("missing 7..8")?;
    /// assert_eq!(*node.value(), "child2");
    ///
    /// let node = tree.node_with_range(8..8).ok_or("missing 8")?;
    /// assert_eq!(*node.value(), "child3");
    ///
    /// let node = tree.node_with_range(8..13).ok_or("missing 9")?;
    /// assert_eq!(*node.value(), "child3");
    ///
    /// let node = tree.node_with_range(2..4).ok_or("missing 2..4")?;
    /// assert_eq!(*node.value(), "root");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Range queries work as expected with checkpoints:
    ///
    /// ```
    /// use syntree::TreeBuilder;
    ///
    /// let mut tree = TreeBuilder::new();
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
    /// assert_eq!(*child.value(), "child");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn node_with_range(&self, span: Range<usize>) -> Option<Node<'_, T, Span>> {
        let start = usize_to_index(span.start)?;
        let end = usize_to_index(span.end)?;
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
    ///     "root2"
    /// };
    ///
    /// let node = tree.node_with_span(Span::point(0)).ok_or("missing 0")?;
    /// assert_eq!(*node.value(), "child1");
    ///
    /// let node = tree.node_with_span(Span::new(0, 3)).ok_or("missing 0")?;
    /// assert_eq!(*node.value(), "child1");
    ///
    /// let node = tree.node_with_span(Span::point(3)).ok_or("missing 3")?;
    /// assert_eq!(*node.value(), "nested1");
    ///
    /// let node = tree.node_with_span(Span::new(3, 7)).ok_or("missing 3..7")?;
    /// assert_eq!(*node.value(), "nested1");
    ///
    /// let node = tree.node_with_span(Span::point(7)).ok_or("missing 7")?;
    /// assert_eq!(*node.value(), "child2");
    ///
    /// let node = tree.node_with_span(Span::new(7, 8)).ok_or("missing 7..8")?;
    /// assert_eq!(*node.value(), "child2");
    ///
    /// let node = tree.node_with_span(Span::point(8)).ok_or("missing 8")?;
    /// assert_eq!(*node.value(), "child3");
    ///
    /// let node = tree.node_with_span(Span::new(8, 13)).ok_or("missing 9")?;
    /// assert_eq!(*node.value(), "child3");
    ///
    /// let node = tree.node_with_span(Span::new(2, 4)).ok_or("missing 2..4")?;
    /// assert_eq!(*node.value(), "root");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Range queries work as expected with checkpoints:
    ///
    /// ```
    /// use syntree::{Span, TreeBuilder};
    ///
    /// let mut tree = TreeBuilder::new();
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
    /// assert_eq!(*child.value(), "child1");
    ///
    /// let child = tree.node_with_span(Span::new(0, 3)).ok_or("missing at 0..3")?;
    /// assert_eq!(*child.value(), "child1");
    ///
    /// let child = tree.node_with_span(Span::new(3, 5)).ok_or("missing at 3..5")?;
    /// assert_eq!(*child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::new(4, 5)).ok_or("missing at 4..5")?;
    /// assert_eq!(*child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::new(3, 4)).ok_or("missing at 3..4")?;
    /// assert_eq!(*child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::point(3)).ok_or("missing at point 5")?;
    /// assert_eq!(*child.value(), "child2");
    ///
    /// let child = tree.node_with_span(Span::new(2, 5)).ok_or("missing at 2..5")?;
    /// assert_eq!(*child.value(), "root");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn node_with_span(&self, span: Span) -> Option<Node<'_, T, Span>> {
        self.node_with_span_internal(span.start, span.end)
    }

    fn node_with_span_internal(&self, start: Index, end: Index) -> Option<Node<'_, T, Span>> {
        let result = self.indexes.binary_search_by(|f| f.index.cmp(&start));

        let n = match result {
            Ok(n) => n.saturating_add(1),
            Err(n) => n,
        };

        let mut node = self.node_at(self.indexes.get(n)?.id)?;

        while let Some(parent) = node.parent() {
            node = parent;

            if parent.span().end >= end {
                break;
            }
        }

        Some(node)
    }
}

impl<T, S> Default for Tree<T, S>
where
    S: span::Builder,
{
    #[inline]
    fn default() -> Self {
        Self::new_with()
    }
}

impl<T, S> PartialEq for Tree<T, S>
where
    T: PartialEq,
    S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.walk().with_depths().eq(other.walk().with_depths())
    }
}

impl<T, S> Eq for Tree<T, S>
where
    T: Eq,
    S: Eq,
{
}

impl<T, S> fmt::Debug for Tree<T, S>
where
    T: fmt::Debug,
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct List<'a, T, S>(&'a Tree<T, S>);

        impl<T, S> fmt::Debug for List<'_, T, S>
        where
            T: fmt::Debug,
            S: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.walk().with_depths()).finish()
            }
        }

        f.debug_tuple("Tree").field(&List(self)).finish()
    }
}
