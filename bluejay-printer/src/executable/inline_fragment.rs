use crate::{directive::DirectivesPrinter, executable::SelectionSetPrinter, write_indent};
use bluejay_core::executable::InlineFragment;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct InlineFragmentPrinter<'a, I: InlineFragment> {
    inline_fragment: &'a I,
    indentation: usize,
}

impl<'a, I: InlineFragment> InlineFragmentPrinter<'a, I> {
    pub(crate) fn new(inline_fragment: &'a I, indentation: usize) -> Self {
        Self {
            inline_fragment,
            indentation,
        }
    }
}

impl<'a, I: InlineFragment> Display for InlineFragmentPrinter<'a, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            inline_fragment,
            indentation,
        } = *self;
        write_indent(f, indentation)?;
        write!(f, "...")?;
        if let Some(type_condition) = inline_fragment.type_condition() {
            write!(f, "on {}", type_condition)?;
        }
        if let Some(directives) = inline_fragment.directives() {
            write!(f, "{}", DirectivesPrinter::new(directives))?;
        }

        write!(
            f,
            " {}",
            SelectionSetPrinter::new(inline_fragment.selection_set(), indentation)
        )
    }
}
