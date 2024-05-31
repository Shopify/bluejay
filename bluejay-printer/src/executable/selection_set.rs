use crate::{executable::SelectionPrinter, write_indent, INDENTATION_SIZE};
use bluejay_core::executable::SelectionSet;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct SelectionSetPrinter<'a, S: SelectionSet> {
    selection_set: &'a S,
    indentation: usize,
}

impl<'a, S: SelectionSet> SelectionSetPrinter<'a, S> {
    pub(crate) fn new(selection_set: &'a S, indentation: usize) -> Self {
        Self {
            selection_set,
            indentation,
        }
    }
}

impl<'a, S: SelectionSet> Display for SelectionSetPrinter<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            selection_set,
            indentation,
        } = *self;
        writeln!(f, "{{")?;
        selection_set.iter().try_for_each(|selection| {
            writeln!(
                f,
                "{}",
                SelectionPrinter::new(selection, indentation + INDENTATION_SIZE)
            )
        })?;
        write_indent(f, indentation)?;
        write!(f, "}}")
    }
}
