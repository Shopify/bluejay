use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::Deref;

pub trait Indexable {
    type Id: Hash + Eq + Ord;

    fn id(&self) -> &Self::Id;
}

pub struct Indexed<'a, T: Indexable>(pub &'a T);

impl<'a, T: Indexable> Clone for Indexed<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: Indexable> Copy for Indexed<'a, T> {}

impl<'a, T: Indexable> From<&'a T> for Indexed<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}

impl<'a, T: Indexable> Hash for Indexed<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id().hash(state);
    }
}

impl<'a, T: Indexable> PartialEq for Indexed<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

impl<'a, T: Indexable> PartialOrd for Indexed<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T: Indexable> Ord for Indexed<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.id().cmp(other.0.id())
    }
}

impl<'a, T: Indexable> Eq for Indexed<'a, T> {}

impl<'a, T: Indexable> Deref for Indexed<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
