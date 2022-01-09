//! Central struct to keep track of all internal linking of a tree.

use crate::non_max::NonMax;
use crate::{Kind, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Links<T> {
    /// The data in the node.
    pub(crate) data: T,
    /// The kind of the node.
    pub(crate) kind: Kind,
    /// Span of the node.
    pub(crate) span: Span,
    /// Parent node. These exists because they are needed when performing range
    /// queries such as [Tree::node_with_range][crate::Tree::node_with_range].
    pub(crate) parent: Option<NonMax>,
    /// The previous node.
    pub(crate) prev: Option<NonMax>,
    /// Next sibling node.
    pub(crate) next: Option<NonMax>,
    /// First child node.
    pub(crate) first: Option<NonMax>,
    /// Last child node.
    pub(crate) last: Option<NonMax>,
}
