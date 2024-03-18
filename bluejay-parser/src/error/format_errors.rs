use ariadne::Source;

struct Index {
    byte: usize,
    char: usize,
}

/// `ariadne` uses char indices, but `logos` uses byte indices.
/// This struct converts byte indices to char indices.
pub(crate) struct ByteIndexToCharIndex<'a> {
    document: &'a str,
    mapping: Vec<Index>,
}

impl<'a> ByteIndexToCharIndex<'a> {
    pub(crate) fn new(document: &'a str) -> Self {
        Self {
            document,
            mapping: vec![Index { byte: 0, char: 0 }],
        }
    }

    pub(crate) fn convert(&mut self, byte_idx: usize) -> usize {
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

    pub(crate) fn convert_span(&mut self, span: &crate::Span) -> std::ops::Range<usize> {
        let span = span.byte_range();
        self.convert(span.start)..self.convert(span.end)
    }
}

pub struct SpanToLocation<'a> {
    byte_idx_to_char_idx: ByteIndexToCharIndex<'a>,
    source: Source<&'a str>,
}

impl<'a> SpanToLocation<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            byte_idx_to_char_idx: ByteIndexToCharIndex::new(s),
            source: Source::from(s),
        }
    }

    pub fn convert(&mut self, span: &crate::Span) -> Option<(usize, usize)> {
        let span = span.byte_range();
        let start = self.byte_idx_to_char_idx.convert(span.start);
        self.source
            .get_offset_line(start)
            .map(|(_, line, col)| (line + 1, col + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::SpanToLocation;
    use crate::Span;

    #[test]
    fn test_span_to_location() {
        let mut span_to_location = SpanToLocation::new("hello\r\nworld");
        assert_eq!(span_to_location.convert(&Span::new(0..5)), Some((1, 1)));
        assert_eq!(span_to_location.convert(&Span::new(7..12)), Some((2, 1)));
    }
}
