use strum::{EnumString, Display, EnumVariantNames, VariantNames};

#[derive(Debug, Clone, Copy, PartialEq, EnumString, Display, EnumVariantNames)]
#[strum(serialize_all = "camelCase")]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    pub const POSSIBLE_VALUES: &'static [&'static str] = Self::VARIANTS;
}
