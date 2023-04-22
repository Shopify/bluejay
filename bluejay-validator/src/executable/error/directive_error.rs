use bluejay_core::executable::ExecutableDocument;
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};

pub enum DirectiveError<'a, const CONST: bool, E: ExecutableDocument> {
    DirectiveDoesNotExist { directive: &'a E::Directive<CONST> },
}

#[cfg(feature = "parser-integration")]
impl<'a, const CONST: bool> From<DirectiveError<'a, CONST, ParserExecutableDocument<'a>>>
    for ParserError
{
    fn from(value: DirectiveError<'a, CONST, ParserExecutableDocument<'a>>) -> Self {
        match value {
            DirectiveError::DirectiveDoesNotExist { directive } => Self::new(
                format!(
                    "No directive definition with name `{}`",
                    directive.name().as_ref()
                ),
                Some(Annotation::new(
                    "No directive definition with this name",
                    directive.name().span().clone(),
                )),
                Vec::new(),
            ),
        }
    }
}
