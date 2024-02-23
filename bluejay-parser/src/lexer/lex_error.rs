use crate::error::{Annotation, Error};
use crate::Span;

#[derive(Debug)]
pub enum LexError {
    UnrecognizedTokenError(Span),
    IntegerValueTooLarge(Span),
    FloatValueTooLarge(Span),
    StringWithInvalidEscapedUnicode(Vec<Span>),
}

impl From<LexError> for Error {
    fn from(val: LexError) -> Self {
        match val {
            LexError::UnrecognizedTokenError(span) => Self::new(
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
            LexError::StringWithInvalidEscapedUnicode(spans) => Self::new(
                "Escaped unicode invalid",
                None,
                spans
                    .into_iter()
                    .map(|span| Annotation::new("Escaped unicode invalid", span))
                    .collect(),
            ),
        }
    }
}
