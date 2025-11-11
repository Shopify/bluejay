use crate::argument::ArgumentsPrinter;
use bluejay_core::{Directive, Directives};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct DirectivePrinter<'a, const CONST: bool, T: Directive<CONST>>(&'a T);

impl<'a, const CONST: bool, T: Directive<CONST>> DirectivePrinter<'a, CONST, T> {
    pub(crate) fn new(directive: &'a T) -> Self {
        Self(directive)
    }
}

impl<const CONST: bool, T: Directive<CONST>> Display for DirectivePrinter<'_, CONST, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(directive) = *self;
        write!(f, "@{}", directive.name())?;
        if let Some(arguments) = directive.arguments() {
            write!(f, "{}", ArgumentsPrinter::new(arguments))?;
        }
        Ok(())
    }
}

pub(crate) struct DirectivesPrinter<'a, const CONST: bool, T: Directives<CONST>>(&'a T);

impl<'a, const CONST: bool, T: Directives<CONST>> DirectivesPrinter<'a, CONST, T> {
    pub(crate) fn new(directives: &'a T) -> Self {
        Self(directives)
    }
}

impl<const CONST: bool, T: Directives<CONST>> Display for DirectivesPrinter<'_, CONST, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(directives) = *self;
        directives
            .iter()
            .try_for_each(|directive| write!(f, " {}", DirectivePrinter::new(directive)))
    }
}

#[cfg(test)]
mod tests {
    use super::DirectivesPrinter;
    use bluejay_parser::ast::{Directives, Parse};

    #[test]
    fn test_directives() {
        let s = " @foo(a: 1, b: 2) @bar";
        let parsed = Directives::<false>::parse(s).result.unwrap();
        assert_eq!(s, DirectivesPrinter::new(&parsed).to_string());
    }
}
