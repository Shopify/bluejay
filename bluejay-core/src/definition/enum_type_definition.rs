use crate::definition::EnumValueDefinitions;
use crate::ConstDirectives;

pub trait EnumTypeDefinition {
    type EnumValueDefinitions: EnumValueDefinitions;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;
    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions;
    fn is_builtin(&self) -> bool;
}
