use crate::EmptyDirectives;
use bluejay_core::executable::FragmentSpread;

pub struct MergedFragmentSpread<'a> {
    name: &'a str,
}

impl<'a> FragmentSpread for MergedFragmentSpread<'a> {
    type Directives = EmptyDirectives<'a>;

    fn name(&self) -> &str {
        self.name
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }
}
