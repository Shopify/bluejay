use std::cmp::{Eq, Ord};
use std::hash::Hash;

pub trait AbstractTypeReference: Sized + Hash + Eq + Ord {
    fn as_ref(&self) -> TypeReference<'_, Self>;
}

#[derive(Debug)]
pub enum TypeReference<'a, T: AbstractTypeReference> {
    NamedType(&'a str, bool),
    ListType(&'a T, bool),
}

impl<'a, T: AbstractTypeReference> Clone for TypeReference<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Self::NamedType(name, required) => Self::NamedType(name, *required),
            Self::ListType(inner, required) => Self::ListType(inner, *required),
        }
    }
}

impl<'a, T: AbstractTypeReference> Copy for TypeReference<'a, T> {}

impl<'a, T: AbstractTypeReference> TypeReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::NamedType(name, _) => name,
            Self::ListType(inner, _) => inner.as_ref().name(),
        }
    }

    pub fn is_required(&self) -> bool {
        match self {
            Self::NamedType(_, required) => *required,
            Self::ListType(_, required) => *required,
        }
    }

    pub fn unwrap_nullable(&self) -> Self {
        match self {
            Self::NamedType(n, _) => Self::NamedType(n, false),
            Self::ListType(l, _) => Self::ListType(l, false),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::NamedType(name, required) => {
                format!("{}{}", name, if *required { "!" } else { "" })
            }
            Self::ListType(inner, required) => {
                format!(
                    "[{}]{}",
                    inner.as_ref().display_name(),
                    if *required { "!" } else { "" }
                )
            }
        }
    }
}
