use crate::executable::SelectionSetPrinter;
use bluejay_core::executable::FragmentDefinition;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct FragmentDefinitionPrinter<'a, T: FragmentDefinition> {
    fragment_definition: &'a T,
}

impl<'a, T: FragmentDefinition> FragmentDefinitionPrinter<'a, T> {
    pub(crate) fn new(fragment_definition: &'a T) -> Self {
        Self {
            fragment_definition,
        }
    }
}

impl<'a, T: FragmentDefinition> Display for FragmentDefinitionPrinter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            fragment_definition,
        } = *self;
        write!(
            f,
            "fragment {} on {} {}",
            fragment_definition.name(),
            fragment_definition.type_condition(),
            SelectionSetPrinter::new(fragment_definition.selection_set(), 0),
        )
    }
}
