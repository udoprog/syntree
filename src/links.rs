//! Central struct to keep track of all internal linking of a tree.

use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Links<T, I, P> {
    /// The data in the node.
    pub(crate) data: T,
    /// Span of the node.
    pub(crate) span: Span<I>,
    /// Parent node. These exists because they are needed when performing range
    /// queries such as [Tree::node_with_range][crate::Tree::node_with_range].
    pub(crate) parent: Option<P>,
    /// The previous node.
    pub(crate) prev: Option<P>,
    /// Next sibling node.
    pub(crate) next: Option<P>,
    /// First child node.
    pub(crate) first: Option<P>,
    /// Last child node.
    pub(crate) last: Option<P>,
}
