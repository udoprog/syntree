//! Central struct to keep track of all internal linking of a tree.

use crate::non_max::NonMax;
use crate::{Kind, Span};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Links<T> {
    /// The data in the node.
    pub(crate) data: T,
    /// The kind of the node.
    pub(crate) kind: Kind,
    /// Span of the node.
    pub(crate) span: Span,
    /// Next sibling node.
    pub(crate) next: Option<NonMax>,
    /// First child node.
    pub(crate) first: Option<NonMax>,
}
