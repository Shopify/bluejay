use crate::{ConstDirectives, Value};
use std::borrow::Cow;

pub trait ScalarTypeDefinition {
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;

    fn coerce_input<const CONST: bool>(
        &self,
        _value: &impl Value<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        Ok(())
    }
}
