use crate::definition::ArgumentsDefinition;
use crate::AsIter;
use strum::{AsRefStr, Display, EnumIter, EnumString, VariantNames};

#[derive(
    Debug, Clone, Copy, EnumString, VariantNames, EnumIter, AsRefStr, Display, PartialEq, Eq,
)]
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

    pub fn is_executable(&self) -> bool {
        matches!(
            self,
            Self::Query
                | Self::Mutation
                | Self::Subscription
                | Self::Field
                | Self::FragmentDefinition
                | Self::FragmentSpread
                | Self::InlineFragment
                | Self::VariableDefinition
        )
    }
}

pub trait DirectiveDefinition {
    type ArgumentsDefinition: ArgumentsDefinition;
    type DirectiveLocations: AsIter<Item = DirectiveLocation>;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition>;
    fn is_repeatable(&self) -> bool;
    fn locations(&self) -> &Self::DirectiveLocations;
    fn is_builtin(&self) -> bool;
}
