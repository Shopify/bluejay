use crate::definition::{HasDirectives, SchemaDefinition};
use crate::Value;
use std::borrow::Cow;

pub trait ScalarTypeDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;

    fn coerce_input<const CONST: bool>(
        &self,
        _value: &impl Value<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        Ok(())
    }
}
