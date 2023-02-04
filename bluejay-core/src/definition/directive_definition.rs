use crate::definition::ArgumentsDefinition;
use crate::AsIter;
use strum::{EnumString, EnumVariantNames, VariantNames};

#[derive(Debug, Clone, Copy, EnumString, EnumVariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

impl DirectiveLocation {
    pub const POSSIBLE_VALUES: &'static [&'static str] = Self::VARIANTS;
}

pub trait DirectiveDefinition {
    type ArgumentsDefinition: ArgumentsDefinition;
    type DirectiveLocations: AsIter<Item = DirectiveLocation>;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition>;
    fn is_repeatable(&self) -> bool;
    fn locations(&self) -> &Self::DirectiveLocations;
}
