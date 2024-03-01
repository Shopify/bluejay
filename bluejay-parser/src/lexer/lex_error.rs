use crate::error::{Annotation, Error};
use crate::Span;

#[derive(Debug)]
pub enum LexError {
    UnrecognizedToken(Span),
    IntegerValueTooLarge(Span),
    FloatValueTooLarge(Span),
    StringValueInvalid(Vec<StringValueLexError>),
}

#[derive(Debug, PartialEq)]
pub enum StringValueLexError {
    InvalidUnicodeEscapeSequence(Span),
    InvalidText(Span),
}

impl From<LexError> for Error {
    fn from(val: LexError) -> Self {
        match val {
            LexError::UnrecognizedToken(span) => Self::new(
                "Unrecognized token",
                Some(Annotation::new("Unable to parse", span)),
                Vec::new(),
            ),
            LexError::IntegerValueTooLarge(span) => Self::new(
                "Value too large to fit in a 32-bit signed integer",
                Some(Annotation::new("Integer too large", span)),
                Vec::new(),
            ),
            LexError::FloatValueTooLarge(span) => Self::new(
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
                            StringValueLexError::InvalidText(span) => ("Invalid text", span),
                        };
                        Annotation::new(message, span)
                    })
                    .collect(),
            ),
        }
    }
}
