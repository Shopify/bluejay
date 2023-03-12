use crate::definition::DirectiveLocation;
use strum::{Display, EnumString, EnumVariantNames, VariantNames};

#[derive(Debug, Clone, Copy, PartialEq, EnumString, Display, EnumVariantNames)]
#[strum(serialize_all = "camelCase")]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    pub const POSSIBLE_VALUES: &'static [&'static str] = Self::VARIANTS;

    pub fn associated_directive_location(&self) -> DirectiveLocation {
        match self {
            Self::Query => DirectiveLocation::Query,
            Self::Mutation => DirectiveLocation::Mutation,
            Self::Subscription => DirectiveLocation::Subscription,
        }
    }
}
