use bluejay_core::{executable::ExecutableDocument, Indexable};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};

pub struct Path<'a, E: ExecutableDocument> {
    root: PathRoot<'a, E>,
    members: Vec<&'a E::Selection>,
}

impl<E: ExecutableDocument> Clone for Path<'_, E> {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            members: self.members.clone(),
        }
    }
}

impl<'a, E: ExecutableDocument> Path<'a, E> {
    pub fn new(root: PathRoot<'a, E>) -> Self {
        Self {
            root,
            members: Vec::new(),
        }
    }

    pub fn root(&self) -> &PathRoot<'a, E> {
        &self.root
    }

    pub fn with_selection(&self, selection: &'a E::Selection) -> Self {
        let mut clone = self.clone();
        clone.members.push(selection);
        clone
    }
}

pub enum PathRoot<'a, E: ExecutableDocument> {
    Operation(&'a E::OperationDefinition),
    Fragment(&'a E::FragmentDefinition),
}

impl<E: ExecutableDocument> Clone for PathRoot<'_, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: ExecutableDocument> Copy for PathRoot<'_, E> {}

impl<E: ExecutableDocument> Hash for PathRoot<'_, E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Operation(o) => o.id().hash(state),
            Self::Fragment(f) => f.id().hash(state),
        }
    }
}

impl<E: ExecutableDocument> PartialEq for PathRoot<'_, E> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Operation(l), Self::Operation(r)) => l.id() == r.id(),
            (Self::Fragment(l), Self::Fragment(r)) => l.id() == r.id(),
            _ => false,
        }
    }
}

impl<E: ExecutableDocument> Eq for PathRoot<'_, E> {}

impl<E: ExecutableDocument> Ord for PathRoot<'_, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Fragment(l), Self::Fragment(r)) => l.id().cmp(r.id()),
            (Self::Fragment(_), Self::Operation(_)) => Ordering::Greater,
            (Self::Operation(_), Self::Fragment(_)) => Ordering::Less,
            (Self::Operation(l), Self::Operation(r)) => l.id().cmp(r.id()),
        }
    }
}

impl<E: ExecutableDocument> PartialOrd for PathRoot<'_, E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
