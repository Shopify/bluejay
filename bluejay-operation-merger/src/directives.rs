use crate::{Error, MergedDirective};
use bluejay_core::{
    definition::DirectiveLocation, executable::ExecutableDocument, AsIter, Directive, Directives,
};
use std::marker::PhantomData;

pub struct EmptyDirectives<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a, const CONST: bool> Directives<CONST> for EmptyDirectives<'a> {
    type Directive = MergedDirective<'a>;
}

impl<'a> AsIter for EmptyDirectives<'a> {
    type Item = MergedDirective<'a>;
    type Iterator<'b> = std::iter::Empty<&'b Self::Item> where Self: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        std::iter::empty()
    }
}

impl<'a> EmptyDirectives<'a> {
    pub(crate) const DEFAULT: Self = Self {
        lifetime: PhantomData,
    };

    pub(crate) fn ensure_empty<const CONST: bool, E: ExecutableDocument>(
        directives: &E::Directives<CONST>,
        location: DirectiveLocation,
    ) -> Result<(), Vec<Error<'_>>> {
        if directives.iter().any(|directive| match directive.name() {
            "suffixOnMerge" => {
                location != DirectiveLocation::Field
                    && location != DirectiveLocation::VariableDefinition
            }
            "replaceOnMerge" => location != DirectiveLocation::VariableDefinition,
            _ => true,
        }) {
            Err(vec![Error::DirectivesNotSupported])
        } else {
            Ok(())
        }
    }
}
