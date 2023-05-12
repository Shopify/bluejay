use crate::ast::definition::CustomScalarTypeDefinition;
use bluejay_core::Value;
use std::borrow::Cow;

pub trait Context: std::fmt::Debug + Sized {
    fn coerce_custom_scalar_input<const CONST: bool>(
        cstd: &CustomScalarTypeDefinition<Self>,
        value: &impl Value<CONST>,
    ) -> Result<(), Cow<'static, str>>;
}

#[derive(Debug)]
pub struct DefaultContext;

impl Context for DefaultContext {
    fn coerce_custom_scalar_input<const CONST: bool>(
        _cstd: &CustomScalarTypeDefinition<Self>,
        _value: &impl Value<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        Ok(())
    }
}
