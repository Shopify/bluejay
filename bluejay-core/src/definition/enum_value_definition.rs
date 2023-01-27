

pub trait EnumValueDefinition {
    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
}
