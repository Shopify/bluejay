use crate::{directive::DirectivesPrinter, value::ValuePrinter};
use bluejay_core::executable::{VariableDefinition, VariableDefinitions, VariableType};

use std::fmt::{Display, Formatter, Result};

pub(crate) struct VariableDefinitionPrinter<'a, T: VariableDefinition> {
    variable_definition: &'a T,
}

impl<'a, T: VariableDefinition> VariableDefinitionPrinter<'a, T> {
    pub(crate) fn new(variable_definition: &'a T) -> Self {
        Self {
            variable_definition,
        }
    }
}

impl<'a, T: VariableDefinition> Display for VariableDefinitionPrinter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            variable_definition,
        } = *self;
        write!(
            f,
            "${}: {}",
            variable_definition.variable(),
            variable_definition.r#type().as_ref().display_name(),
        )?;
        if let Some(default_value) = variable_definition.default_value() {
            write!(f, " = {}", ValuePrinter::new(default_value))?;
        }

        if let Some(directives) =  variable_definition.directives() {
            write!(
                f,
                "{}",
                DirectivesPrinter::new(directives)
            )?;
        };
        Ok(())
    }
}

pub(crate) struct VariableDefinitionsPrinter<'a, T: VariableDefinitions> {
    variable_definitions: &'a T,
}

impl<'a, T: VariableDefinitions> VariableDefinitionsPrinter<'a, T> {
    pub(crate) fn new(variable_definitions: &'a T) -> Self {
        Self {
            variable_definitions,
        }
    }
}

impl<'a, T: VariableDefinitions> Display for VariableDefinitionsPrinter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            variable_definitions,
        } = *self;
        if !variable_definitions.is_empty() {
            write!(f, "(")?;
            variable_definitions.iter().enumerate().try_for_each(
                |(idx, variable_definition)| {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", VariableDefinitionPrinter::new(variable_definition))
                },
            )?;
            write!(f, ")")?;
        }
        Ok(())
    }
}
