use crate::{AbstractValue, ConstDirectives};
use std::borrow::Cow;

pub trait ScalarTypeDefinition {
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;

    fn coerce_input<const CONST: bool>(
        &self,
        _value: &impl AbstractValue<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        Ok(())
    }
}
