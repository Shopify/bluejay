#[cfg(feature = "format-errors")]
use ariadne::{Config, IndexType, Label, Report, ReportKind, Source};
use std::borrow::Cow;

mod annotation;
#[cfg(feature = "format-errors")]
mod format_errors;

pub use annotation::Annotation;
#[cfg(feature = "format-errors")]
pub use format_errors::SpanToLocation;

#[derive(Debug, PartialEq)]
pub struct Error {
    message: Cow<'static, str>,
    primary_annotation: Option<Annotation>,
    secondary_annotations: Vec<Annotation>,
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
