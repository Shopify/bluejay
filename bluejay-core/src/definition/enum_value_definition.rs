use crate::definition::HasDirectives;

pub trait EnumValueDefinition: HasDirectives {
    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
}
