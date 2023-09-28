use crate::definition::Directives;

pub trait HasDirectives {
    type Directives: Directives;

    fn directives(&self) -> Option<&Self::Directives>;
}
