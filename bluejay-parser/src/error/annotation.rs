use crate::Span;

#[derive(Debug, PartialEq)]
pub struct Annotation {
    pub message: String,
    pub span: Span,
}
