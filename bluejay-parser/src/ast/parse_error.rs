use crate::error::{Annotation, AnnotationType, Error};
use crate::Span;

#[derive(Debug)]
pub enum ParseError {
    ExpectedOneOf {
        span: Span,
        values: &'static [&'static str],
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

impl From<ParseError> for Error {
    fn from(val: ParseError) -> Self {
        match val {
            ParseError::ExpectedOneOf { span, values } => Self {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: format!("Expected one of the following: {}", values.join(", "),),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            ParseError::ExpectedIdentifier { span, value } => Self {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: format!("Expected to find: {value}"),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            ParseError::ExpectedName { span } => Self {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: "Expected a name".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            ParseError::UnexpectedEOF { span } => Self {
                message: "Parse error".to_string(),
                annotations: vec![Annotation {
                    message: "Unexpected EOF".to_string(),
                    annotation_type: AnnotationType::Primary,
                    span,
                }],
            },
            ParseError::UnexpectedToken { span } => Self {
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
