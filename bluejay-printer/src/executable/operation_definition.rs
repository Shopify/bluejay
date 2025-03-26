use crate::{
    directive::DirectivesPrinter,
    executable::{SelectionSetPrinter, VariableDefinitionsPrinter},
};
use bluejay_core::executable::OperationDefinition;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct OperationDefinitionPrinter<'a, O: OperationDefinition> {
    operation_definition: &'a O,
}

impl<'a, O: OperationDefinition> OperationDefinitionPrinter<'a, O> {
    pub(crate) fn new(operation_definition: &'a O) -> Self {
        Self {
            operation_definition,
        }
    }
}

impl<O: OperationDefinition> Display for OperationDefinitionPrinter<'_, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            operation_definition,
        } = *self;
        let operation_definition_reference = operation_definition.as_ref();
        write!(f, "{}", operation_definition_reference.operation_type())?;
        if let Some(name) = operation_definition_reference.name() {
            write!(f, " {}", name)?;
        }
        if let Some(variable_definitions) = operation_definition_reference.variable_definitions() {
            write!(
                f,
                "{}",
                VariableDefinitionsPrinter::new(variable_definitions)
            )?;
        }
        if let Some(directives) = operation_definition_reference.directives() {
            write!(f, "{}", DirectivesPrinter::new(directives))?;
        }
        write!(
            f,
            " {}",
            SelectionSetPrinter::new(operation_definition_reference.selection_set(), 0)
        )
    }
}
