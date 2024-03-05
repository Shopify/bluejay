use crate::error::{Annotation, Error};
use crate::Span;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexError {
    #[default]
    UnrecognizedToken,
    IntegerValueTooLarge,
    FloatValueTooLarge,
    StringValueInvalid(Vec<StringValueLexError>),
}

impl From<Vec<StringValueLexError>> for LexError {
    fn from(errors: Vec<StringValueLexError>) -> Self {
        Self::StringValueInvalid(errors)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringValueLexError {
    InvalidUnicodeEscapeSequence(Span),
    InvalidCharacters(Span),
}

impl From<(LexError, Span)> for Error {
    fn from((error, span): (LexError, Span)) -> Self {
        match error {
            LexError::UnrecognizedToken => Self::new(
                "Unrecognized token",
                Some(Annotation::new("Unable to parse", span)),
                Vec::new(),
            ),
            LexError::IntegerValueTooLarge => Self::new(
                "Value too large to fit in a 32-bit signed integer",
                Some(Annotation::new("Integer too large", span)),
                Vec::new(),
            ),
            LexError::FloatValueTooLarge => Self::new(
                "Value too large to fit in a 64-bit float",
                Some(Annotation::new("Float too large", span)),
                Vec::new(),
            ),
            LexError::StringValueInvalid(errors) => Self::new(
                "String value invalid",
                None,
                errors
                    .into_iter()
                    .map(|error| {
                        let (message, span) = match error {
                            StringValueLexError::InvalidUnicodeEscapeSequence(span) => {
                                ("Invalid unicode escape sequence", span)
                            }
                            StringValueLexError::InvalidCharacters(span) => {
                                ("Invalid characters", span)
                            }
                        };
                        Annotation::new(message, span)
                    })
                    .collect(),
            ),
        }
    }
}
