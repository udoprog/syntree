//! Building trees in every feature configuration, including without `alloc`.

use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::slice;

use syntree::{Builder, Error, Storage, TreeIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Full;

/// Storage holding at most `N` items inline, without allocating.
struct ArrayVec<T, const N: usize> {
    len: usize,
    data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        // SAFETY: the first `len` items are initialized.
        unsafe { slice::from_raw_parts(self.data.as_ptr().cast(), self.len) }
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: the first `len` items are initialized.
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.len) }
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        // SAFETY: the first `len` items are initialized and dropped only here.
        unsafe { core::ptr::drop_in_place(&mut **self) }
    }
}

impl<T, const N: usize> Storage<T> for ArrayVec<T, N> {
    type Error = Full;

    const EMPTY: Self = ArrayVec {
        len: 0,
        data: [const { MaybeUninit::uninit() }; N],
    };

    fn with_capacity(capacity: usize) -> Result<Self, Full> {
        if capacity > N {
            return Err(Full);
        }

        Ok(Self::EMPTY)
    }

    fn capacity(&self) -> usize {
        N
    }

    fn push(&mut self, item: T) -> Result<(), Full> {
        let Some(slot) = self.data.get_mut(self.len) else {
            return Err(Full);
        };

        slot.write(item);
        self.len += 1;
        Ok(())
    }
}

syntree::flavor! {
    struct FlavorArray {
        type Index = u32;
        type Error = Full;
        type Storage<T> = ArrayVec<T, 4>;
        type Indexes = ArrayVec<TreeIndex<Self>, 4>;
    }
}

#[test]
fn custom_storage_builds_tree() -> Result<(), Error<Full>> {
    let mut tree: Builder<_, FlavorArray> = Builder::new_with();

    tree.open("root")?;
    tree.token("a", 2)?;
    tree.token("b", 3)?;
    tree.close()?;

    let tree = tree.build()?;

    let root = tree.first().ok_or(Error::MissingNode(0))?;
    assert_eq!(root.value(), "root");
    assert_eq!(root.span().range(), 0..5);

    let tokens = root.children().map(|n| n.value()).collect::<Vec<_>>();
    assert_eq!(tokens, ["a", "b"]);
    Ok(())
}

#[test]
fn custom_storage_reports_full() -> Result<(), Error<Full>> {
    let mut tree: Builder<_, FlavorArray> = Builder::new_with();

    for _ in 0..4 {
        tree.token("a", 1)?;
    }

    assert_eq!(tree.token("a", 1), Err(Error::Flavor(Full)));
    Ok(())
}

#[test]
#[cfg(any(feature = "alloc", feature = "std"))]
fn default_flavor_builds_tree() -> Result<(), Box<dyn core::error::Error>> {
    let mut tree = Builder::new();

    tree.open("root")?;
    tree.token("a", 2)?;
    tree.close()?;

    let tree = tree.build()?;

    let expected = syntree::tree! {
        "root" => {
            ("a", 2)
        }
    };

    assert_eq!(tree, expected);
    Ok(())
}
