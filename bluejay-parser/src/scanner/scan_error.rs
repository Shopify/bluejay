use crate::error::{Annotation, Error};
use crate::Span;

#[derive(Debug)]
pub enum ScanError {
    UnrecognizedTokenError(Span),
    IntegerValueTooLarge(Span),
    FloatValueTooLarge(Span),
    StringWithInvalidEscapedUnicode(Vec<Span>),
}

impl From<ScanError> for Error {
    fn from(val: ScanError) -> Self {
        match val {
            ScanError::UnrecognizedTokenError(span) => Self::new(
                "Unrecognized token",
                Some(Annotation::new("Unable to parse", span)),
                Vec::new(),
            ),
            ScanError::IntegerValueTooLarge(span) => Self::new(
                "Value too large to fit in a 32-bit signed integer",
                Some(Annotation::new("Integer too large", span)),
                Vec::new(),
            ),
            ScanError::FloatValueTooLarge(span) => Self::new(
                "Value too large to fit in a 64-bit float",
                Some(Annotation::new("Float too large", span)),
                Vec::new(),
            ),
            ScanError::StringWithInvalidEscapedUnicode(spans) => Self::new(
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
