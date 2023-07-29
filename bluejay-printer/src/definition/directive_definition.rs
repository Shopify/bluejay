use crate::{
    definition::arguments_definition::ArgumentsDefinitionPrinter,
    string_value::BlockStringValuePrinter,
};
use bluejay_core::{definition::DirectiveDefinition, AsIter};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct DirectiveDefinitionPrinter<'a, D: DirectiveDefinition>(&'a D);

impl<'a, D: DirectiveDefinition> DirectiveDefinitionPrinter<'a, D> {
    pub(crate) fn new(directive_definition: &'a D) -> Self {
        Self(directive_definition)
    }
}

impl<'a, D: DirectiveDefinition> Display for DirectiveDefinitionPrinter<'a, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(directive_definition) = *self;
        if let Some(description) = directive_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "directive @{}", directive_definition.name())?;

        if let Some(arguments_definition) = directive_definition.arguments_definition() {
            write!(
                f,
                "{}",
                ArgumentsDefinitionPrinter::new(arguments_definition, 0)
            )?;
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

        writeln!(f)
    }
}
