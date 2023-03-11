use crate::error::{Annotation, Error};
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
    EmptyDocument,
}

impl From<ParseError> for Error {
    fn from(val: ParseError) -> Self {
        match val {
            ParseError::ExpectedOneOf { span, values } => Self {
                message: "Parse error".to_string(),
                primary_annotation: Some(Annotation {
                    message: format!("Expected one of the following: {}", values.join(", "),),
                    span,
                }),
                secondary_annotations: Vec::new(),
            },
            ParseError::ExpectedIdentifier { span, value } => Self {
                message: "Parse error".to_string(),
                primary_annotation: Some(Annotation {
                    message: format!("Expected to find: {value}"),
                    span,
                }),
                secondary_annotations: Vec::new(),
            },
            ParseError::ExpectedName { span } => Self {
                message: "Parse error".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Expected a name".to_string(),
                    span,
                }),
                secondary_annotations: Vec::new(),
            },
            ParseError::UnexpectedEOF { span } => Self {
                message: "Parse error".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Unexpected EOF".to_string(),
                    span,
                }),
                secondary_annotations: Vec::new(),
            },
            ParseError::UnexpectedToken { span } => Self {
                message: "Unexpected token".to_string(),
                primary_annotation: Some(Annotation {
                    message: "Unexpected token".to_string(),
                    span,
                }),
                secondary_annotations: Vec::new(),
            },
            ParseError::EmptyDocument => Self {
                message: "Document does not contain any definitions".to_string(),
                primary_annotation: None,
                secondary_annotations: Vec::new(),
            },
        }
    }
}
