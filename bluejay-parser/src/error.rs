use ariadne::{Config, IndexType, Label, Report, ReportKind, Source};
use itertools::Either;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;

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
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

/// A [spec compliant GraphQL Error](https://spec.graphql.org/draft/#sec-Errors.Error-Result-Format)
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
        let converter = RefCell::new(SpanToLocation::new(document));
        errors
            .into_iter()
            .flat_map(|err| {
                let err: Error = err.into();
                if let Some(primary_annotation) = err.primary_annotation {
                    let (line, col) = converter
                        .borrow_mut()
                        .convert(primary_annotation.span())
                        .unwrap_or((0, 0));
                    Either::Left(std::iter::once(GraphQLError {
                        message: primary_annotation.message,
                        locations: vec![Location { line, col }],
                    }))
                } else if !err.secondary_annotations.is_empty() {
                    Either::Right(err.secondary_annotations.into_iter().map(
                        |secondary_annotation| {
                            let (line, col) = converter
                                .borrow_mut()
                                .convert(secondary_annotation.span())
                                .unwrap_or((0, 0));
                            GraphQLError {
                                message: secondary_annotation.message,
                                locations: vec![Location { line, col }],
                            }
                        },
                    ))
                } else {
                    Either::Left(std::iter::once(GraphQLError {
                        message: err.message,
                        locations: vec![],
                    }))
                }
            })
            .collect()
    }

    #[cfg(feature = "format-errors")]
    pub fn format_errors<E: Into<Error>>(
        document: &str,
        filename: Option<&str>,
        errors: impl IntoIterator<Item = E>,
    ) -> String {
        let filename = filename.unwrap_or("<unknown>");
        let mut file_cache = (filename, Source::from(document));

        let mut buf: Vec<u8> = Vec::new();

        errors
            .into_iter()
            .enumerate()
            .try_for_each(|(idx, error)| {
                let error: Error = error.into();
                if idx != 0 {
                    buf.extend("\n".as_bytes());
                }
                Report::<(&str, logos::Span)>::build(
                    ReportKind::Error,
                    (
                        filename,
                        error
                            .primary_annotation
                            .as_ref()
                            .map(|a| a.span().clone().into())
                            .unwrap_or(0..0),
                    ),
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
                            Label::new((filename, span.into()))
                                .with_message(message.as_ref())
                                .with_priority(1)
                        }),
                )
                .with_labels(error.secondary_annotations.into_iter().map(
                    |Annotation { message, span }| {
                        Label::new((filename, span.into())).with_message(message.as_ref())
                    },
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
