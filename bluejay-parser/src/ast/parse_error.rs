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
        }
    }
}
