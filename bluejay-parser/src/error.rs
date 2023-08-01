#[cfg(feature = "format-errors")]
use ariadne::{Config, Label, Report, ReportKind, Source};
use std::borrow::Cow;

mod annotation;

pub use annotation::Annotation;

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
        let mut byte_idx_to_char_idx = ByteIndexToCharIndex::new(document);

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
                        .map(|a| byte_idx_to_char_idx.convert_span(a.span()).start)
                        .unwrap_or(0),
                )
                .with_config(Config::default().with_color(false))
                .with_message(error.message)
                .with_labels(error.primary_annotation.map(|annotation| {
                    Label::new(byte_idx_to_char_idx.convert_span(annotation.span()))
                        .with_message(annotation.message())
                        .with_priority(1)
                }))
                .with_labels(error.secondary_annotations.into_iter().map(|annotation| {
                    Label::new(byte_idx_to_char_idx.convert_span(annotation.span()))
                        .with_message(annotation.message())
                }))
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

struct Index {
    byte: usize,
    char: usize,
}

/// `ariadne` uses char indices, but `logos` uses byte indices.
/// This struct converts byte indices to char indices.
struct ByteIndexToCharIndex<'a> {
    document: &'a str,
    mapping: Vec<Index>,
}

impl<'a> ByteIndexToCharIndex<'a> {
    fn new(document: &'a str) -> Self {
        Self {
            document,
            mapping: vec![Index { byte: 0, char: 0 }],
        }
    }

    fn convert(&mut self, byte_idx: usize) -> usize {
        match self
            .mapping
            .binary_search_by_key(&byte_idx, |index| index.byte)
        {
            Ok(idx) => self.mapping[idx].char,
            Err(idx) => {
                let char_idx = self.mapping[idx - 1].char
                    + self.document[self.mapping[idx - 1].byte..byte_idx]
                        .chars()
                        .count();
                self.mapping.insert(
                    idx,
                    Index {
                        byte: byte_idx,
                        char: char_idx,
                    },
                );
                char_idx
            }
        }
    }

    fn convert_span(&mut self, span: &crate::Span) -> std::ops::Range<usize> {
        let span = span.byte_range();
        self.convert(span.start)..self.convert(span.end)
    }
}
