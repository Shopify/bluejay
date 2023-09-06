use std::cmp::{Eq, Ord};
use std::hash::Hash;

pub trait VariableType: Sized + Hash + Eq + Ord {
    fn as_ref(&self) -> VariableTypeReference<'_, Self>;
}

#[derive(Debug)]
pub enum VariableTypeReference<'a, T: VariableType> {
    Named(&'a str, bool),
    List(&'a T, bool),
}

impl<'a, T: VariableType> Clone for VariableTypeReference<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: VariableType> Copy for VariableTypeReference<'a, T> {}

impl<'a, T: VariableType> VariableTypeReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::Named(name, _) => name,
            Self::List(inner, _) => inner.as_ref().name(),
        }
    }

    pub fn is_required(&self) -> bool {
        match self {
            Self::Named(_, required) => *required,
            Self::List(_, required) => *required,
        }
    }

    pub fn unwrap_nullable(&self) -> Self {
        match self {
            Self::Named(n, _) => Self::Named(n, false),
            Self::List(l, _) => Self::List(l, false),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Named(name, required) => {
                format!("{}{}", name, if *required { "!" } else { "" })
            }
            Self::List(inner, required) => {
                format!(
                    "[{}]{}",
                    inner.as_ref().display_name(),
                    if *required { "!" } else { "" }
                )
            }
        }
    }
}
