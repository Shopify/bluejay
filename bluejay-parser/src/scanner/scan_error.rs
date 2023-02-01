use crate::error::{Annotation, AnnotationType, Error};
use crate::Span;

#[derive(Debug)]
pub enum ScanError {
    UnrecognizedTokenError(Span),
    IntegerValueTooLarge(Span),
    FloatValueTooLarge(Span),
}

impl From<ScanError> for Error {
    fn from(val: ScanError) -> Self {
        match val {
            ScanError::UnrecognizedTokenError(span) => Self {
                message: "Unrecognized token".to_string(),
                annotations: vec![Annotation {
                    message: "Unable to parse".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            ScanError::IntegerValueTooLarge(span) => Self {
                message: "Value too large to fit in a 32-bit signed integer".to_string(),
                annotations: vec![Annotation {
                    message: "Integer too large".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            ScanError::FloatValueTooLarge(span) => Self {
                message: "Value too large to fit in a 64-bit float".to_string(),
                annotations: vec![Annotation {
                    message: "Float too large".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
        }
    }
}
