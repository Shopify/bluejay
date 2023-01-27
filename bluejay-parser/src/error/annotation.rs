use crate::Span;

#[derive(Debug, PartialEq)]
pub enum AnnotationType {
    Primary,
    Secondary,
}

#[derive(Debug, PartialEq)]
pub struct Annotation {
    pub message: String,
    pub annotation_type: AnnotationType,
    pub span: Span,
}
