use crate::value::DisplayValue;
use bluejay_core::{Argument, Arguments};
use std::fmt::{Error, Write};

pub(crate) struct DisplayArgument;

impl DisplayArgument {
    pub(crate) fn fmt<const CONST: bool, T: Argument<CONST>, W: Write>(
        argument: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        write!(f, "{}: ", argument.name())?;
        DisplayValue::fmt(argument.value().as_ref(), f)
    }
}

pub(crate) struct DisplayArguments;

impl DisplayArguments {
    pub(crate) fn fmt<const CONST: bool, T: Arguments<CONST>, W: Write>(
        arguments: &T,
        f: &mut W,
    ) -> Result<(), Error> {
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
                DisplayArgument::fmt(argument, f)
            })?;
        write!(f, ")")
    }

    #[cfg(test)]
    fn to_string<const CONST: bool, T: Arguments<CONST>>(arguments: &T) -> String {
        let mut s = String::new();
        Self::fmt(arguments, &mut s).expect("fmt returned an error unexpectedly");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayArguments;
    use bluejay_parser::ast::{Arguments, Parse};

    #[test]
    fn test_arguments() {
        let s = "(a: 1, b: 2)";
        let parsed = Arguments::<false>::parse(s).unwrap();
        assert_eq!(s, DisplayArguments::to_string(&parsed));
    }
}
