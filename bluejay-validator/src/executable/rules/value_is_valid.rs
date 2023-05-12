use crate::executable::{Cache, Error, Path, Rule, Visitor};
use crate::value::input_coercion::CoerceInput;
use bluejay_core::definition::{InputValueDefinition, SchemaDefinition};
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};
use bluejay_core::Argument;

pub struct ValueIsValid<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for ValueIsValid<'a, E, S>
{
    fn visit_variable_definition(
        &mut self,
        variable_definition: &'a <E as ExecutableDocument>::VariableDefinition,
    ) {
        if let Some(default_value) = variable_definition.default_value() {
            if let Some(input_value_definition) = self
                .cache
                .variable_definition_input_type(variable_definition.r#type())
            {
                if let Err(coercion_errors) =
                    input_value_definition.coerce_value(default_value, &[])
                {
                    self.errors
                        .extend(coercion_errors.into_iter().map(Error::InvalidConstValue));
                }
            }
        }
    }

    fn visit_const_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<true>,
        input_value_definition: &'a <S as SchemaDefinition>::InputValueDefinition,
    ) {
        if let Err(coercion_errors) = input_value_definition
            .r#type()
            .coerce_value(argument.value(), &[])
        {
            self.errors
                .extend(coercion_errors.into_iter().map(Error::InvalidConstValue));
        }
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<false>,
        input_value_definition: &'a <S as SchemaDefinition>::InputValueDefinition,
        _: &Path<'a, E>,
    ) {
        if let Err(coercion_errors) = input_value_definition
            .r#type()
            .coerce_value(argument.value(), &[])
        {
            self.errors
                .extend(coercion_errors.into_iter().map(Error::InvalidVariableValue));
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for ValueIsValid<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for ValueIsValid<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            cache,
        }
    }
}
