use crate::{
    definition::arguments_definition::ArgumentsDefinitionPrinter, directive::DirectivesPrinter,
    string_value::BlockStringValuePrinter, write_indent, INDENTATION_SIZE,
};
use bluejay_core::{
    definition::{FieldDefinition, FieldsDefinition, OutputType},
    AsIter,
};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct FieldDefinitionPrinter<'a, F: FieldDefinition> {
    field_definition: &'a F,
    indentation: usize,
}

impl<'a, F: FieldDefinition> FieldDefinitionPrinter<'a, F> {
    pub(crate) fn new(field_definition: &'a F, indentation: usize) -> Self {
        Self {
            field_definition,
            indentation,
        }
    }
}

impl<'a, F: FieldDefinition> Display for FieldDefinitionPrinter<'a, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            field_definition,
            indentation,
        } = *self;
        if let Some(description) = field_definition.description() {
            write!(
                f,
                "{}",
                BlockStringValuePrinter::new(description, indentation)
            )?;
        }

        write_indent(f, indentation)?;
        write!(f, "{}", field_definition.name(),)?;

        if let Some(arguments_definition) = field_definition.arguments_definition() {
            write!(
                f,
                "{}",
                ArgumentsDefinitionPrinter::new(arguments_definition, indentation)
            )?;
        }

        write!(f, ": {}", field_definition.r#type().as_ref().display_name())?;

        if let Some(directives) = field_definition.directives() {
            if !directives.is_empty() {
                write!(f, " {}", DirectivesPrinter::new(directives))?;
            }
        }

        writeln!(f)
    }
}

pub(crate) struct FieldsDefinitionPrinter<'a, F: FieldsDefinition> {
    fields_definition: &'a F,
    indentation: usize,
}

impl<'a, F: FieldsDefinition> FieldsDefinitionPrinter<'a, F> {
    pub(crate) fn new(fields_definition: &'a F, indentation: usize) -> Self {
        Self {
            fields_definition,
            indentation,
        }
    }
}

impl<'a, F: FieldsDefinition> Display for FieldsDefinitionPrinter<'a, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            fields_definition,
            indentation,
        } = *self;
        writeln!(f, "{{")?;

        fields_definition
            .iter()
            .filter(|fd| !fd.is_builtin())
            .enumerate()
            .try_for_each(|(idx, fd)| {
                if idx != 0 {
                    writeln!(f)?;
                }
                FieldDefinitionPrinter::new(fd, indentation + INDENTATION_SIZE).fmt(f)
            })?;

        write_indent(f, indentation)?;
        writeln!(f, "}}")
    }
}
