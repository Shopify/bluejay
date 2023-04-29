use bluejay_core::definition::{DirectiveDefinition, DirectiveLocation, SchemaDefinition};
use bluejay_core::executable::ExecutableDocument;
use bluejay_core::AsIter;
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};
use itertools::Itertools;

pub enum DirectiveError<'a, const CONST: bool, E: ExecutableDocument, S: SchemaDefinition> {
    DirectiveDoesNotExist {
        directive: &'a E::Directive<CONST>,
    },
    DirectiveInInvalidLocation {
        directive: &'a E::Directive<CONST>,
        directive_definition: &'a S::DirectiveDefinition,
        location: DirectiveLocation,
    },
}

#[cfg(feature = "parser-integration")]
impl<'a, const CONST: bool, S: SchemaDefinition>
    From<DirectiveError<'a, CONST, ParserExecutableDocument<'a>, S>> for ParserError
{
    fn from(value: DirectiveError<'a, CONST, ParserExecutableDocument<'a>, S>) -> Self {
        match value {
            DirectiveError::DirectiveDoesNotExist { directive } => Self::new(
                format!(
                    "No directive definition with name `@{}`",
                    directive.name().as_ref()
                ),
                Some(Annotation::new(
                    "No directive definition with this name",
                    directive.name().span().clone(),
                )),
                Vec::new(),
            ),
            DirectiveError::DirectiveInInvalidLocation {
                directive,
                directive_definition,
                location,
            } => Self::new(
                format!(
                    "Directive @{} cannot be used at location {location}. It is only allowed at the following locations: {}",
                    directive.name().as_ref(),
                    directive_definition.locations().iter().join(", "),
                ),
                Some(Annotation::new(
                    format!("Cannot be used at location {location}"),
                    directive.span().clone(),
                )),
                Vec::new(),
            ),
        }
    }
}
