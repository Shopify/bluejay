use crate::{
    definition::input_value_definition::InputValueDefinitionPrinter, directive::DirectivesPrinter,
    string_value::BlockStringValuePrinter, INDENTATION_SIZE,
};
use bluejay_core::{definition::InputObjectTypeDefinition, AsIter};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct InputObjectTypeDefinitionPrinter<'a, I: InputObjectTypeDefinition>(&'a I);

impl<'a, I: InputObjectTypeDefinition> InputObjectTypeDefinitionPrinter<'a, I> {
    pub(crate) fn new(input_object_type_definition: &'a I) -> Self {
        Self(input_object_type_definition)
    }
}

impl<'a, I: InputObjectTypeDefinition> Display for InputObjectTypeDefinitionPrinter<'a, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(input_object_type_definition) = *self;
        if let Some(description) = input_object_type_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "input {}", input_object_type_definition.name())?;

        if let Some(directives) = input_object_type_definition.directives() {
            write!(f, "{}", DirectivesPrinter::new(directives))?;
        }

        writeln!(f, " {{")?;

        input_object_type_definition
            .input_field_definitions()
            .iter()
            .enumerate()
            .try_for_each(|(idx, ivd)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                write!(
                    f,
                    "{}",
                    InputValueDefinitionPrinter::new(ivd, INDENTATION_SIZE)
                )
            })?;

        writeln!(f, "}}")
    }
}
