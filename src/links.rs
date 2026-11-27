//! The storage entries of a tree.
//!
//! A tree stores two flat arrays:
//!
//! * Tokens, in source order. A token stores only its end, its start is the
//!   end of the token before it (or zero for the first one). Where an explicit
//!   span leaves a gap after the previous token, a gap entry is stored in front
//!   of the token to carry the gap's start. Gap entries are never visible
//!   through the public API.
//! * Interior nodes, which cover a range of the token array and link only to
//!   other interior nodes.
//!
//! A token's parent pointer also carries its trivia flag in its top bit, see
//! [`Pointer::flagged`], so trivia costs no space.
//!
//! The children of a node are its tokens whose parent is the node, merged in
//! token order with its chain of interior children. An interior child comes
//! before the token at index `i` if the child's token range starts at or before
//! `i`.

use core::cell::Cell;

use crate::pointer::Pointer;
use crate::span::Span;

/// A token entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TokenLinks<T, I, P>
where
    T: Copy,
{
    /// The data in the token.
    pub(crate) data: Cell<T>,
    /// The end of the token. Its start is the end of the entry before it.
    pub(crate) end: I,
    /// The interior node the token belongs to, `None` at the root, with the
    /// trivia flag in its top bit. A gap entry holds [`Pointer::GAP`], which
    /// no node is ever given. Read through [`TokenLinks::parent`] and
    /// [`TokenLinks::is_trivia`].
    link: Option<P>,
}

impl<T, I, P> TokenLinks<T, I, P>
where
    T: Copy,
    P: Pointer,
{
    /// A token entry.
    #[inline]
    pub(crate) fn token(data: T, end: I, parent: Option<P>, trivia: bool) -> Self {
        Self {
            data: Cell::new(data),
            end,
            link: if trivia { P::flagged(parent) } else { parent },
        }
    }

    /// A gap entry, ending where the token after it starts.
    #[inline]
    pub(crate) fn gap(data: T, end: I) -> Self {
        Self {
            data: Cell::new(data),
            end,
            link: Some(P::GAP),
        }
    }

    /// Test if this is a gap entry.
    #[inline]
    pub(crate) fn is_gap(&self) -> bool {
        self.link == Some(P::GAP)
    }

    /// Test if this is a trivia token. Gap entries are not.
    #[inline]
    pub(crate) fn is_trivia(&self) -> bool {
        !self.is_gap() && P::is_flagged(self.link)
    }

    /// The interior node the token belongs to, `None` at the root.
    #[inline]
    pub(crate) fn parent(&self) -> Option<P> {
        P::unflagged(self.link)
    }

    /// Move the token to another parent, keeping its trivia flag.
    #[inline]
    pub(crate) fn set_parent(&mut self, parent: Option<P>) {
        self.link = if self.is_trivia() {
            P::flagged(parent)
        } else {
            parent
        };
    }
}

/// An interior node entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NodeLinks<T, I, P>
where
    T: Copy,
{
    /// The data in the node.
    pub(crate) data: Cell<T>,
    /// Span of the node.
    pub(crate) span: Span<I>,
    /// The start of the token range covered by the node, as a token index.
    pub(crate) tok_start: P,
    /// The end of the token range covered by the node, exclusive. Only set
    /// once the node is closed.
    pub(crate) tok_end: P,
    /// Parent node.
    pub(crate) parent: Option<P>,
    /// The previous interior sibling.
    pub(crate) prev: Option<P>,
    /// The next interior sibling.
    pub(crate) next: Option<P>,
    /// First interior child.
    pub(crate) first: Option<P>,
    /// Last interior child.
    pub(crate) last: Option<P>,
}

// Entries should be no larger than their fields, give or take alignment
// padding.
#[test]
fn test_size() {
    use crate::pointer::Width;
    use core::mem::size_of;

    type P<W> = <W as Width>::Pointer;

    assert_eq!(size_of::<TokenLinks<u16, u32, P<u32>>>(), 12);
    assert_eq!(size_of::<NodeLinks<u16, u32, P<u32>>>(), 40);
    assert_eq!(size_of::<TokenLinks<u16, u32, P<usize>>>(), 16);
    assert_eq!(size_of::<NodeLinks<u16, u32, P<usize>>>(), 72);
}
