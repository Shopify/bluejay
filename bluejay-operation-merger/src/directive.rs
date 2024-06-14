use crate::{MergedArguments, Never};
use bluejay_core::Directive;
use std::marker::PhantomData;

pub struct MergedDirective<'a> {
    lifetime: PhantomData<&'a ()>,
    _never: Never,
}

impl<'a, const CONST: bool> Directive<CONST> for MergedDirective<'a> {
    type Arguments = MergedArguments<'a, CONST>;

    fn name(&self) -> &str {
        unreachable!()
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        unreachable!()
    }
}
