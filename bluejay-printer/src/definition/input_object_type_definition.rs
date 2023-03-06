use crate::{
    definition::input_value_definition::DisplayInputValueDefinition, directive::DisplayDirectives,
    string_value::DisplayStringValue, INDENTATION_SIZE,
};
use bluejay_core::{definition::InputObjectTypeDefinition, AsIter};
use std::fmt::{Error, Write};

pub(crate) struct DisplayInputObjectTypeDefinition;

impl DisplayInputObjectTypeDefinition {
    pub(crate) fn fmt<T: InputObjectTypeDefinition, W: Write>(
        input_object_type_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = input_object_type_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "input {}", input_object_type_definition.name())?;

        if let Some(directives) = input_object_type_definition.directives() {
            if !directives.is_empty() {
                write!(f, " ")?;
                DisplayDirectives::fmt(directives, f)?;
            }
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
                DisplayInputValueDefinition::fmt(ivd, f, INDENTATION_SIZE)
            })?;

        writeln!(f, "}}")
    }
}
