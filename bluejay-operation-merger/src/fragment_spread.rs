use crate::EmptyDirectives;
use bluejay_core::executable::{ExecutableDocument, FragmentSpread};
use std::marker::PhantomData;

pub struct MergedFragmentSpread<'a, E: ExecutableDocument> {
    name: &'a str,
    executable_document_type: PhantomData<E>,
}

impl<'a, E: ExecutableDocument> FragmentSpread for MergedFragmentSpread<'a, E> {
    type Directives = EmptyDirectives<false, E>;

    fn name(&self) -> &str {
        self.name
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }
}
