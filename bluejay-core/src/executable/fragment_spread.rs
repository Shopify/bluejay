use crate::VariableDirectives;

pub trait FragmentSpread {
    type Directives: VariableDirectives;

    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;
}
