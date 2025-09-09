use crate::error::{Annotation, Error};
use crate::Span;

#[derive(Debug)]
pub enum ParseError {
    InvalidEnumValue {
        span: Span,
        value: String,
    },
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
    MaxDepthExceeded,
    MaxTokensExceeded {
        span: Span,
        limit: usize,
    },
}

impl From<ParseError> for Error {
    fn from(val: ParseError) -> Self {
        match val {
            ParseError::InvalidEnumValue { span, value } => Self::new(
                "Parse error",
                Some(Annotation::new(
                    format!("{value} is not an allowed enum value"),
                    span,
                )),
                Vec::new(),
            ),
            ParseError::ExpectedOneOf { span, values } => Self::new(
                "Parse error",
                Some(Annotation::new(
                    format!("Expected one of the following: {}", values.join(", "),),
                    span,
                )),
                Vec::new(),
            ),
            ParseError::ExpectedIdentifier { span, value } => Self::new(
                "Parse error",
                Some(Annotation::new(format!("Expected to find: {value}"), span)),
                Vec::new(),
            ),
            ParseError::ExpectedName { span } => Self::new(
                "Parse error",
                Some(Annotation::new("Expected a name", span)),
                Vec::new(),
            ),
            ParseError::UnexpectedEOF { span } => Self::new(
                "Parse error",
                Some(Annotation::new("Unexpected EOF", span)),
                Vec::new(),
            ),
            ParseError::UnexpectedToken { span } => Self::new(
                "Unexpected token",
                Some(Annotation::new("Unexpected token", span)),
                Vec::new(),
            ),
            ParseError::EmptyDocument => Self::new(
                "Document does not contain any definitions",
                None,
                Vec::new(),
            ),
            ParseError::MaxDepthExceeded => Self::new("Max depth exceeded", None, Vec::new()),
            ParseError::MaxTokensExceeded { span, limit } => Self::new(
                "Max tokens exceeded",
                Some(Annotation::new(
                    format!("Maximum token limit of {limit} exceeded"),
                    span,
                )),
                Vec::new(),
            ),
        }
    }
}
