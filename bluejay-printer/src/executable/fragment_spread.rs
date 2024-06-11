use bluejay_core::executable::FragmentSpread;
use std::fmt::{Display, Formatter, Result};

use crate::{directive::DirectivesPrinter, write_indent};

pub(crate) struct FragmentSpreadPrinter<'a, T: FragmentSpread> {
    fragment_spread: &'a T,
    indentation: usize,
}

impl<'a, T: FragmentSpread> FragmentSpreadPrinter<'a, T> {
    pub(crate) fn new(fragment_spread: &'a T, indentation: usize) -> Self {
        Self {
            fragment_spread,
            indentation,
        }
    }
}

impl<'a, T: FragmentSpread> Display for FragmentSpreadPrinter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            fragment_spread,
            indentation,
        } = *self;
        write_indent(f, indentation)?;
        write!(
            f,
            "...{}{}",
            fragment_spread.name(),
            DirectivesPrinter::new(fragment_spread.directives())
        )
    }
}
