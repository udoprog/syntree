use core::ops::{Deref, DerefMut};

use syntree::{Builder, Error, Storage, TreeIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Full;

/// Storage which refuses to hold more than two items.
struct Bounded<T>(Vec<T>);

impl<T> Bounded<T> {
    const LIMIT: usize = 2;
}

impl<T> Deref for Bounded<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T> DerefMut for Bounded<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T> Storage<T> for Bounded<T> {
    type Error = Full;

    const EMPTY: Self = Bounded(Vec::new());

    fn with_capacity(capacity: usize) -> Result<Self, Full> {
        if capacity > Self::LIMIT {
            return Err(Full);
        }

        Ok(Bounded(Vec::with_capacity(capacity)))
    }

    fn capacity(&self) -> usize {
        Self::LIMIT
    }

    fn push(&mut self, item: T) -> Result<(), Full> {
        if self.0.len() == Self::LIMIT {
            return Err(Full);
        }

        self.0.push(item);
        Ok(())
    }
}

syntree::flavor! {
    struct FlavorBounded {
        type Index = u32;
        type Error = Full;
        type Storage<T> = Bounded<T>;
        type Indexes = Bounded<TreeIndex<Self>>;
    }
}

#[test]
fn custom_storage_and_error() {
    let mut b: Builder<&str, FlavorBounded> = Builder::new_with();
    b.open("root").unwrap();
    b.token("a", 1).unwrap();
    assert_eq!(b.token("b", 1), Err(Error::Flavor(Full)));
}
