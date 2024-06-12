#[cfg(feature = "format-errors")]
use ariadne::{Config, IndexType, Label, Report, ReportKind, Source};
use std::borrow::Cow;

mod annotation;
mod format_errors;

pub use annotation::Annotation;
pub use format_errors::SpanToLocation;

#[derive(Debug, PartialEq)]
pub struct Error {
    message: Cow<'static, str>,
    primary_annotation: Option<Annotation>,
    secondary_annotations: Vec<Annotation>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

/// A [spec compliant GraphQL Error](https://spec.graphql.org/draft/#sec-Errors.Error-Result-Format)
#[derive(Debug, PartialEq, Eq)]
pub struct GraphQLError {
    pub message: Cow<'static, str>,
    pub locations: Vec<Location>,
}

impl Error {
    pub fn new(
        message: impl Into<Cow<'static, str>>,
        primary_annotation: Option<Annotation>,
        secondary_annotations: Vec<Annotation>,
    ) -> Self {
        Self {
            message: message.into(),
            primary_annotation,
            secondary_annotations,
        }
    }

    pub fn into_graphql_errors<E: Into<Error>>(
        document: &str,
        errors: impl IntoIterator<Item = E>,
    ) -> Vec<GraphQLError> {
        let mut converter = SpanToLocation::new(document);
        errors
            .into_iter()
            .flat_map(|err| {
                let err: Error = err.into();
                if let Some(ref primary_annotation) = err.primary_annotation {
                    let (line, col) = converter
                        .convert(primary_annotation.span())
                        .unwrap_or((0, 0));
                    return vec![GraphQLError {
                        message: primary_annotation.message.clone(),
                        locations: vec![Location { line, col }],
                    }];
                }

                err.secondary_annotations
                    .iter()
                    .map(|secondary_annotation| {
                        let (line, col) = converter
                            .convert(secondary_annotation.span())
                            .unwrap_or((0, 0));
                        GraphQLError {
                            message: secondary_annotation.message.clone(),
                            locations: vec![Location { line, col }],
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[cfg(feature = "format-errors")]
    pub fn format_errors<E: Into<Error>>(
        document: &str,
        errors: impl IntoIterator<Item = E>,
    ) -> String {
        let mut file_cache = Source::from(document);

        let mut buf: Vec<u8> = Vec::new();

        errors
            .into_iter()
            .enumerate()
            .try_for_each(|(idx, error)| {
                let error: Error = error.into();
                if idx != 0 {
                    buf.extend("\n".as_bytes());
                }
                Report::build(
                    ReportKind::Error,
                    (),
                    error
                        .primary_annotation
                        .as_ref()
                        .map(|a| a.span().byte_range().start)
                        .unwrap_or(0),
                )
                .with_config(
                    Config::default()
                        .with_color(false)
                        .with_index_type(IndexType::Byte),
                )
                .with_message(error.message)
                .with_labels(
                    error
                        .primary_annotation
                        .map(|Annotation { message, span }| {
                            Label::new(span)
                                .with_message(message.as_ref())
                                .with_priority(1)
                        }),
                )
                .with_labels(error.secondary_annotations.into_iter().map(
                    |Annotation { message, span }| Label::new(span).with_message(message.as_ref()),
                ))
                .finish()
                .write(&mut file_cache, &mut buf)
            })
            .unwrap();

        String::from_utf8(buf).unwrap()
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }
}
