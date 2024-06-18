use crate::{
    argument::ArgumentsPrinter, directive::DirectivesPrinter, executable::SelectionSetPrinter,
    write_indent,
};
use bluejay_core::executable::Field;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct FieldPrinter<'a, F: Field> {
    field: &'a F,
    indentation: usize,
}

impl<'a, F: Field> FieldPrinter<'a, F> {
    pub(crate) fn new(field: &'a F, indentation: usize) -> Self {
        Self { field, indentation }
    }
}

impl<'a, F: Field> Display for FieldPrinter<'a, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { field, indentation } = *self;
        write_indent(f, indentation)?;
        if let Some(alias) = field.alias() {
            write!(f, "{}: ", alias)?;
        }
        write!(f, "{}", field.name())?;
        if let Some(arguments) = field.arguments() {
            write!(f, "{}", ArgumentsPrinter::new(arguments))?;
        }
        if let Some(directives) = field.directives() {
            write!(f, "{}", DirectivesPrinter::new(directives))?;
        }
        if let Some(selection_set) = field.selection_set() {
            write!(
                f,
                " {}",
                SelectionSetPrinter::new(selection_set, indentation)
            )?;
        }
        Ok(())
    }
}
