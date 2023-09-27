use crate::definition::{EnumValueDefinitions, HasDirectives};

pub trait EnumTypeDefinition: HasDirectives {
    type EnumValueDefinitions: EnumValueDefinitions;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions;
    fn is_builtin(&self) -> bool;
}
