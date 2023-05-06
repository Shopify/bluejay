use bluejay_core::executable::ExecutableDocument;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};

pub struct Path<'a, E: ExecutableDocument> {
    root: PathRoot<'a, E>,
    members: Vec<&'a E::Selection>,
}

impl<'a, E: ExecutableDocument> Clone for Path<'a, E> {
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

impl<'a, E: ExecutableDocument> Clone for PathRoot<'a, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Operation(o) => Self::Operation(o),
            Self::Fragment(f) => Self::Fragment(f),
        }
    }
}

impl<'a, E: ExecutableDocument> Copy for PathRoot<'a, E> {}

impl<'a, E: ExecutableDocument> Hash for PathRoot<'a, E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Operation(o) => o.hash(state),
            Self::Fragment(f) => f.hash(state),
        }
    }
}

impl<'a, E: ExecutableDocument> PartialEq for PathRoot<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Operation(l), Self::Operation(r)) if l == r => true,
            (Self::Fragment(l), Self::Fragment(r)) if l == r => true,
            _ => false,
        }
    }
}

impl<'a, E: ExecutableDocument> Eq for PathRoot<'a, E> {}

impl<'a, E: ExecutableDocument> Ord for PathRoot<'a, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Fragment(l), Self::Fragment(r)) => l.cmp(r),
            (Self::Fragment(_), Self::Operation(_)) => Ordering::Greater,
            (Self::Operation(_), Self::Fragment(_)) => Ordering::Less,
            (Self::Operation(l), Self::Operation(r)) => l.cmp(r),
        }
    }
}

impl<'a, E: ExecutableDocument> PartialOrd for PathRoot<'a, E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
