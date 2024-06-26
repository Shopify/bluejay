use crate::executable::{
    document::{Error, Path, Rule, Visitor},
    Cache,
};
use crate::value::input_coercion::CoerceInput;
use bluejay_core::definition::{InputValueDefinition, SchemaDefinition};
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};
use bluejay_core::Argument;

pub struct ValueIsValid<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for ValueIsValid<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
            cache,
        }
    }

    fn visit_variable_definition(
        &mut self,
        variable_definition: &'a <E as ExecutableDocument>::VariableDefinition,
    ) {
        if let Some(default_value) = variable_definition.default_value() {
            if let Some(input_value_definition) = self
                .cache
                .variable_definition_input_type(variable_definition.r#type())
            {
                if let Err(coercion_errors) = self.schema_definition.coerce_value(
                    input_value_definition,
                    default_value,
                    Default::default(),
                ) {
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
        if let Err(coercion_errors) = self.schema_definition.coerce_value(
            input_value_definition.r#type(),
            argument.value(),
            Default::default(),
        ) {
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
        if let Err(coercion_errors) = self.schema_definition.coerce_value(
            input_value_definition.r#type(),
            argument.value(),
            Default::default(),
        ) {
            self.errors
                .extend(coercion_errors.into_iter().map(Error::InvalidVariableValue));
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for ValueIsValid<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
