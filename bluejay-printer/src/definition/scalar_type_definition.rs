use crate::{directive::DirectivesPrinter, string_value::BlockStringValuePrinter};
use bluejay_core::{definition::ScalarTypeDefinition, AsIter};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct ScalarTypeDefinitionPrinter<'a, S: ScalarTypeDefinition>(&'a S);

impl<'a, S: ScalarTypeDefinition> ScalarTypeDefinitionPrinter<'a, S> {
    pub(crate) fn new(scalar_type_definition: &'a S) -> Self {
        Self(scalar_type_definition)
    }
}

impl<'a, S: ScalarTypeDefinition> Display for ScalarTypeDefinitionPrinter<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(scalar_type_definition) = *self;
        if let Some(description) = scalar_type_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "scalar {}", scalar_type_definition.name())?;

        if let Some(directives) = scalar_type_definition.directives() {
            if !directives.is_empty() {
                write!(f, " {}", DirectivesPrinter::new(directives))?;
            }
        }

        writeln!(f)
    }
}
