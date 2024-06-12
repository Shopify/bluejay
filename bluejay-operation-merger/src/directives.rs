use crate::Error;
use bluejay_core::{executable::ExecutableDocument, AsIter, Directives};
use std::marker::PhantomData;

pub struct EmptyDirectives<const CONST: bool, E: ExecutableDocument> {
    directive_type: PhantomData<E>,
}

impl<const CONST: bool, E: ExecutableDocument> Directives<CONST> for EmptyDirectives<CONST, E> {
    type Directive = E::Directive<CONST>;
}

impl<const CONST: bool, E: ExecutableDocument> AsIter for EmptyDirectives<CONST, E> {
    type Item = E::Directive<CONST>;
    type Iterator<'a> = std::iter::Empty<&'a Self::Item> where E: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        std::iter::empty()
    }
}

impl<const CONST: bool, E: ExecutableDocument> EmptyDirectives<CONST, E> {
    pub(crate) const DEFAULT: Self = Self {
        directive_type: PhantomData,
    };

    pub(crate) fn ensure_empty(directives: &E::Directives<CONST>) -> Result<(), Vec<Error<'_, E>>> {
        if directives.iter().next().is_some() {
            Err(vec![Error::DirectivesNotSupported])
        } else {
            Ok(())
        }
    }
}
