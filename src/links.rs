//! Central struct to keep track of all internal linking of a tree.

use core::cell::Cell;

use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Links<T, I, P>
where
    T: Copy,
{
    /// The data in the node.
    pub(crate) data: Cell<T>,
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

// These tests might not always pass, due to alignment. But it's nice to ensure
#[test]
fn test_size() {
    macro_rules! test {
        ($data:ty, $index:ty, $width:ty, $max_align:expr) => {
            assert!(
                (
                    std::mem::size_of::<Links<$data, $index, <$width as crate::pointer::Width>::Pointer>>() as isize
                    -
                    (std::mem::size_of::<$data>() as isize + ((<$index>::BITS * 2) / 8) as isize + ((<$width>::BITS * 5) / 8) as isize)
                ).abs() <= $max_align
            );
        }
    }

    test!([u8; 8], u32, u16, 4);
    test!([u8; 8], u32, u32, 4);
    test!([u8; 8], u32, u64, 4);
    test!([u8; 8], u32, u128, 4);
}
