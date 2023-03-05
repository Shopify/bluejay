use crate::{
    definition::arguments_definition::DisplayArgumentsDefinition, string_value::DisplayStringValue,
};
use bluejay_core::{definition::DirectiveDefinition, AsIter};
use std::fmt::{Error, Write};

pub(crate) struct DisplayDirectiveDefinition;

impl DisplayDirectiveDefinition {
    pub(crate) fn fmt<T: DirectiveDefinition, W: Write>(
        directive_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = directive_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "directive @{} ", directive_definition.name())?;

        if let Some(arguments_definition) = directive_definition.arguments_definition() {
            DisplayArgumentsDefinition::fmt(arguments_definition, f, 0)?;
        }

        if directive_definition.is_repeatable() {
            write!(f, " repeatable")?;
        }

        write!(f, " on ")?;

        directive_definition
            .locations()
            .iter()
            .enumerate()
            .try_for_each(|(idx, location)| {
                if idx != 0 {
                    write!(f, " | ")?;
                }
                write!(f, "{location}")
            })?;

        writeln!(f)?;
        writeln!(f)
    }
}
