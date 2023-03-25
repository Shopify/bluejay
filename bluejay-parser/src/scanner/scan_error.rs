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
            ScanError::UnrecognizedTokenError(span) => Self {
                message: "Unrecognized token".to_string(),
                primary_annotation: Some(Annotation::new("Unable to parse", span)),
                secondary_annotations: Vec::new(),
            },
            ScanError::IntegerValueTooLarge(span) => Self {
                message: "Value too large to fit in a 32-bit signed integer".to_string(),
                primary_annotation: Some(Annotation::new("Integer too large", span)),
                secondary_annotations: Vec::new(),
            },
            ScanError::FloatValueTooLarge(span) => Self {
                message: "Value too large to fit in a 64-bit float".to_string(),
                primary_annotation: Some(Annotation::new("Float too large", span)),
                secondary_annotations: Vec::new(),
            },
            ScanError::StringWithInvalidEscapedUnicode(spans) => Self {
                message: "Escaped unicode invalid".to_string(),
                primary_annotation: None,
                secondary_annotations: spans
                    .into_iter()
                    .map(|span| Annotation::new("Escaped unicode invalid", span))
                    .collect(),
            },
        }
    }
}
