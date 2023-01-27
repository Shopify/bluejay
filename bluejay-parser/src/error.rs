mod annotation;

pub use annotation::{Annotation, AnnotationType};

#[derive(Debug, PartialEq)]
pub struct Error {
    pub message: String,
    pub annotations: Vec<Annotation>,
}
