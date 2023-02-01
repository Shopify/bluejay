use crate::error::{Annotation, AnnotationType, Error};
use crate::Span;

#[derive(Debug)]
pub enum ParseError {
    ExpectedOneOf {
        span: Span,
        values: Vec<&'static str>,
    },
    ExpectedIdentifier {
        span: Span,
        value: String,
    },
    ExpectedName {
        span: Span,
    },
    UnexpectedEOF {
        span: Span,
    },
    UnexpectedToken {
        span: Span,
    },
}

impl Into<Error> for ParseError {
    fn into(self) -> Error {
        match self {
            Self::ExpectedOneOf { span, values } => Error {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: format!("Expected one of the following: {}", values.join(", "),),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            Self::ExpectedIdentifier { span, value } => Error {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: format!("Expected to find: {}", value),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            Self::ExpectedName { span } => Error {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: "Expected a name".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            Self::UnexpectedEOF { span } => Error {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: "Unexpected EOF".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            Self::UnexpectedToken { span } => Error {
                message: "Unexpected token".to_string(),
                annotations: vec![Annotation {
                    message: "Unexpected token".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
        }
    }
}
