use crate::Span;
use crate::error::{Error, Annotation, AnnotationType};

#[derive(Debug)]
pub enum ScanError {
    UnrecognizedTokenError(Span),
    IntegerValueTooLarge(Span),
    FloatValueTooLarge(Span),
}

impl Into<Error> for ScanError {
    fn into(self) -> Error {
        match self {
            Self::UnrecognizedTokenError(span) => Error {
                message: "Unrecognized token".to_string(),
                annotations: vec![
                    Annotation {
                        message: "Unable to parse".to_string(),
                        annotation_type: AnnotationType::Primary,
                        span,
                    }
                ]
            },
            Self::IntegerValueTooLarge(span) => Error {
                message: "Value too large to fit in a 32-bit signed integer".to_string(),
                annotations: vec![
                    Annotation {
                        message: "Integer too large".to_string(),
                        annotation_type: AnnotationType::Primary,
                        span,
                    }
                ]
            },
            Self::FloatValueTooLarge(span) => Error {
                message: "Value too large to fit in a 64-bit float".to_string(),
                annotations: vec![
                    Annotation {
                        message: "Float too large".to_string(),
                        annotation_type: AnnotationType::Primary,
                        span,
                    }
                ]
            },
        }
    }
}
