use crate::argument::DisplayArguments;
use bluejay_core::{Directive, Directives};
use std::fmt::{Error, Write};

pub(crate) struct DisplayDirective;

impl DisplayDirective {
    pub(crate) fn fmt<const CONST: bool, T: Directive<CONST>, W: Write>(
        directive: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        write!(f, "@{}", directive.name())?;
        if let Some(arguments) = directive.arguments() {
            DisplayArguments::fmt(arguments, f)?;
        }
        Ok(())
    }
}

pub(crate) struct DisplayDirectives;

impl DisplayDirectives {
    pub(crate) fn fmt<const CONST: bool, T: Directives<CONST>, W: Write>(
        directives: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        directives
            .iter()
            .enumerate()
            .try_for_each(|(idx, directive)| {
                if idx != 0 {
                    write!(f, " ")?;
                }
                DisplayDirective::fmt(directive, f)
            })
    }

    #[cfg(test)]
    fn to_string<const CONST: bool, T: Directives<CONST>>(directives: &T) -> String {
        let mut s = String::new();
        Self::fmt(directives, &mut s).expect("fmt returned an error unexpectedly");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayDirectives;
    use bluejay_parser::ast::{Directives, Parse};

    #[test]
    fn test_directives() {
        let s = "@foo(a: 1, b: 2) @bar";
        let parsed = Directives::<false>::parse(s).unwrap();
        assert_eq!(s, DisplayDirectives::to_string(&parsed));
    }
}
