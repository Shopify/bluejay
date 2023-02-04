use crate::ConstDirectives;

pub trait EnumValueDefinition {
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;
}
