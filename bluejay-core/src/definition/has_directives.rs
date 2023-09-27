use crate::directive::Directives;
pub trait HasDirectives {
    type Directives: Directives<true>;

    fn directives(&self) -> Option<&Self::Directives>;
}
