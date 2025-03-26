use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::Deref;

pub trait Indexable {
    type Id: Hash + Eq + Ord;

    fn id(&self) -> &Self::Id;
}

pub struct Indexed<'a, T: Indexable>(pub &'a T);

impl<T: Indexable> Clone for Indexed<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Indexable> Copy for Indexed<'_, T> {}

impl<'a, T: Indexable> From<&'a T> for Indexed<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}

impl<T: Indexable> Hash for Indexed<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id().hash(state);
    }
}

impl<T: Indexable> PartialEq for Indexed<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

impl<T: Indexable> PartialOrd for Indexed<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Indexable> Ord for Indexed<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.id().cmp(other.0.id())
    }
}

impl<T: Indexable> Eq for Indexed<'_, T> {}

impl<T: Indexable> Deref for Indexed<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
