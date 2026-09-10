use crate::definition::{HasDirectives, InputType, SchemaDefinition};
use crate::ConstValue;

pub trait InputValueDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;
    type Value: ConstValue;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn r#type(&self) -> &<Self::SchemaDefinition as SchemaDefinition>::InputType;
    fn default_value(&self) -> Option<&Self::Value>;

    fn is_required(&self) -> bool {
        self.default_value().is_none() && self.r#type().is_required()
    }
}
