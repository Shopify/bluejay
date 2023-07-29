use crate::value::ValuePrinter;
use bluejay_core::{Argument, Arguments};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct ArgumentPrinter<'a, const CONST: bool, T: Argument<CONST>>(&'a T);

impl<'a, const CONST: bool, T: Argument<CONST>> ArgumentPrinter<'a, CONST, T> {
    pub(crate) fn new(argument: &'a T) -> Self {
        Self(argument)
    }
}

impl<'a, const CONST: bool, T: Argument<CONST>> Display for ArgumentPrinter<'a, CONST, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(argument) = *self;
        write!(
            f,
            "{}: {}",
            argument.name(),
            ValuePrinter::new(argument.value())
        )
    }
}

pub(crate) struct ArgumentsPrinter<'a, const CONST: bool, T: Arguments<CONST>>(&'a T);

impl<'a, const CONST: bool, T: Arguments<CONST>> ArgumentsPrinter<'a, CONST, T> {
    pub(crate) fn new(arguments: &'a T) -> Self {
        Self(arguments)
    }
}

impl<'a, const CONST: bool, T: Arguments<CONST>> Display for ArgumentsPrinter<'a, CONST, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(arguments) = *self;
        if arguments.is_empty() {
            return Ok(());
        }
        write!(f, "(")?;
        arguments
            .iter()
            .enumerate()
            .try_for_each(|(idx, argument)| {
                if idx != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", ArgumentPrinter::new(argument))
            })?;
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::ArgumentsPrinter;
    use bluejay_parser::ast::{Arguments, Parse};

    #[test]
    fn test_arguments() {
        let s = "(a: 1, b: 2)";
        let parsed = Arguments::<false>::parse(s).unwrap();
        assert_eq!(s, ArgumentsPrinter::new(&parsed).to_string());
    }
}
