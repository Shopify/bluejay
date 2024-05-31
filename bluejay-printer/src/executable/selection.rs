use crate::executable::{FieldPrinter, FragmentSpreadPrinter, InlineFragmentPrinter};
use bluejay_core::executable::{Selection, SelectionReference};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct SelectionPrinter<'a, S: Selection> {
    selection: &'a S,
    indentation: usize,
}

impl<'a, S: Selection> SelectionPrinter<'a, S> {
    pub(crate) fn new(selection: &'a S, indentation: usize) -> Self {
        Self {
            selection,
            indentation,
        }
    }
}

impl<'a, S: Selection> Display for SelectionPrinter<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            selection,
            indentation,
        } = *self;
        match selection.as_ref() {
            SelectionReference::Field(field) => {
                write!(f, "{}", FieldPrinter::new(field, indentation))
            }
            SelectionReference::FragmentSpread(fragment_spread) => {
                write!(
                    f,
                    "{}",
                    FragmentSpreadPrinter::new(fragment_spread, indentation)
                )
            }
            SelectionReference::InlineFragment(inline_fragment) => write!(
                f,
                "{}",
                InlineFragmentPrinter::new(inline_fragment, indentation)
            ),
        }
    }
}
