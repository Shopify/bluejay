use crate::executable::{FragmentDefinitionPrinter, OperationDefinitionPrinter};
use bluejay_core::executable::ExecutableDocument;
use std::fmt::{Display, Formatter, Result};

pub struct ExecutableDocumentPrinter<'a, T: ExecutableDocument> {
    executable_document: &'a T,
}

impl<'a, T: ExecutableDocument> ExecutableDocumentPrinter<'a, T> {
    pub fn new(executable_document: &'a T) -> Self {
        Self {
            executable_document,
        }
    }

    pub fn to_string(executable_document: &'a T) -> String {
        Self::new(executable_document).to_string()
    }
}

impl<T: ExecutableDocument> Display for ExecutableDocumentPrinter<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            executable_document,
        } = *self;
        executable_document
            .operation_definitions()
            .enumerate()
            .try_for_each(|(idx, operation_definition)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                writeln!(
                    f,
                    "{}",
                    OperationDefinitionPrinter::new(operation_definition)
                )
            })?;

        executable_document
            .fragment_definitions()
            .try_for_each(|fragment_definition| {
                writeln!(f)?;
                writeln!(f, "{}", FragmentDefinitionPrinter::new(fragment_definition))
            })
    }
}
