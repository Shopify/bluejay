use crate::definition::EnumValueDefinitions;

pub trait EnumTypeDefinition {
    type EnumValueDefinitions: EnumValueDefinitions;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions;
}
