use bluejay_core::definition::{
    DirectiveDefinition, FieldDefinition, InputValueDefinition, SchemaDefinition,
};
use bluejay_core::executable::ExecutableDocument;
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};
use itertools::Itertools;

pub enum ArgumentError<'a, const CONST: bool, E: ExecutableDocument, S: SchemaDefinition> {
    NonUniqueArgumentNames {
        arguments: Vec<&'a E::Argument<CONST>>,
        name: &'a str,
    },
    ArgumentDoesNotExistOnField {
        argument: &'a E::Argument<CONST>,
        field_definition: &'a S::FieldDefinition,
    },
    ArgumentDoesNotExistOnDirective {
        argument: &'a E::Argument<CONST>,
        directive_definition: &'a S::DirectiveDefinition,
    },
    DirectiveMissingRequiredArguments {
        directive: &'a E::Directive<CONST>,
        directive_definition: &'a S::DirectiveDefinition,
        missing_argument_definitions: Vec<&'a S::InputValueDefinition>,
    },
    FieldMissingRequiredArguments {
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        missing_argument_definitions: Vec<&'a S::InputValueDefinition>,
    },
}

#[cfg(feature = "parser-integration")]
impl<'a, const CONST: bool, S: SchemaDefinition>
    From<ArgumentError<'a, CONST, ParserExecutableDocument<'a>, S>> for ParserError
{
    fn from(value: ArgumentError<'a, CONST, ParserExecutableDocument<'a>, S>) -> Self {
        match value {
            ArgumentError::NonUniqueArgumentNames { arguments, name } => Self::new(
                format!("Multiple arguments with name `{name}`"),
                None,
                arguments
                    .into_iter()
                    .map(|argument| {
                        Annotation::new(
                            format!("Argument with name `{name}`"),
                            argument.name().span().clone(),
                        )
                    })
                    .collect(),
            ),
            ArgumentError::ArgumentDoesNotExistOnField {
                argument,
                field_definition,
            } => Self::new(
                format!(
                    "Field `{}` does not define an argument named `{}`",
                    field_definition.name(),
                    argument.name().as_ref(),
                ),
                Some(Annotation::new(
                    "No argument definition with this name",
                    argument.name().span().clone(),
                )),
                Vec::new(),
            ),
            ArgumentError::ArgumentDoesNotExistOnDirective {
                argument,
                directive_definition,
            } => Self::new(
                format!(
                    "Directive `{}` does not define an argument named `{}`",
                    directive_definition.name(),
                    argument.name().as_ref(),
                ),
                Some(Annotation::new(
                    "No argument definition with this name",
                    argument.name().span().clone(),
                )),
                Vec::new(),
            ),
            ArgumentError::DirectiveMissingRequiredArguments {
                directive,
                missing_argument_definitions,
                ..
            } => {
                let missing_argument_names = missing_argument_definitions
                    .into_iter()
                    .map(InputValueDefinition::name)
                    .join(", ");
                Self::new(
                    format!(
                        "Directive `{}` missing argument(s): {missing_argument_names}",
                        directive.name().as_ref(),
                    ),
                    Some(Annotation::new(
                        format!("Missing argument(s): {missing_argument_names}"),
                        directive.span().clone(),
                    )),
                    Vec::new(),
                )
            }
            ArgumentError::FieldMissingRequiredArguments {
                field,
                field_definition: _,
                missing_argument_definitions,
            } => {
                let missing_argument_names = missing_argument_definitions
                    .into_iter()
                    .map(InputValueDefinition::name)
                    .join(", ");
                let span = match field.arguments() {
                    Some(arguments) => field.name().span().merge(arguments.span()),
                    None => field.name().span().clone(),
                };
                Self::new(
                    format!(
                        "Field `{}` missing argument(s): {missing_argument_names}",
                        field.response_key()
                    ),
                    Some(Annotation::new(
                        format!("Missing argument(s): {missing_argument_names}"),
                        span,
                    )),
                    Vec::new(),
                )
            }
        }
    }
}
