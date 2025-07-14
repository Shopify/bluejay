use std::borrow::Cow;

/// An optimized string builder that minimizes allocations
///
/// Key optimizations:
/// 1. For strings without escapes, returns the original borrowed string
/// 2. For strings with escapes, uses a single allocation sized appropriately
/// 3. Tracks whether we've seen escape sequences to choose the right path
pub struct CowStringBuilder<'a> {
    /// The original source string slice (used for no-escape case)
    source: Option<&'a str>,
    /// Buffer for building strings with escape sequences
    buffer: Option<String>,
    /// Tracks the state of string building
    state: BuilderState,
}

#[derive(Debug, Clone, Copy)]
enum BuilderState {
    /// Initial state - no content yet
    Empty,
    /// We've only seen source characters (no escapes)
    SourceOnly,
    /// We've seen escape sequences
    HasEscapes,
}

impl<'a> CowStringBuilder<'a> {
    #[inline]
    pub fn new(_estimated_size: usize) -> Self {
        Self {
            source: None,
            buffer: None,
            state: BuilderState::Empty,
        }
    }

    #[inline]
    pub fn append_source(&mut self, s: &'a str) {
        match self.state {
            BuilderState::Empty => {
                // First content - save reference for potential zero-copy return
                self.source = Some(s);
                self.state = BuilderState::SourceOnly;
            }
            BuilderState::SourceOnly => {
                // We have multiple source chunks or non-contiguous source
                // Need to switch to buffer mode
                self.state = BuilderState::HasEscapes;
                let mut buffer = String::with_capacity(self.source.unwrap().len() + s.len() + 32);
                buffer.push_str(self.source.unwrap());
                buffer.push_str(s);
                self.buffer = Some(buffer);
            }
            BuilderState::HasEscapes => {
                // Already using buffer, just append
                self.buffer.as_mut().unwrap().push_str(s);
            }
        }
    }

    #[inline]
    pub fn append_char(&mut self, c: char) {
        // Any escaped character forces us to use the buffer
        match self.state {
            BuilderState::Empty => {
                self.state = BuilderState::HasEscapes;
                let mut buffer = String::with_capacity(32);
                buffer.push(c);
                self.buffer = Some(buffer);
            }
            BuilderState::SourceOnly => {
                // Switch from source-only to buffer mode
                self.state = BuilderState::HasEscapes;
                let source = self.source.unwrap();
                let mut buffer = String::with_capacity(source.len() + 32);
                buffer.push_str(source);
                buffer.push(c);
                self.buffer = Some(buffer);
            }
            BuilderState::HasEscapes => {
                self.buffer.as_mut().unwrap().push(c);
            }
        }
    }

    #[inline]
    pub fn finish(self) -> Cow<'a, str> {
        match self.state {
            BuilderState::Empty => Cow::Borrowed(""),
            BuilderState::SourceOnly => {
                // Perfect case - no allocations needed!
                Cow::Borrowed(self.source.unwrap())
            }
            BuilderState::HasEscapes => {
                // Had to allocate due to escapes
                Cow::Owned(self.buffer.unwrap())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_escapes() {
        let mut builder = CowStringBuilder::new(0);
        builder.append_source("Hello, world!");
        let result = builder.finish();
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_with_escapes() {
        let mut builder = CowStringBuilder::new(0);
        builder.append_source("Hello, ");
        builder.append_char('\\');
        builder.append_char('n');
        builder.append_source("world!");
        let result = builder.finish();
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "Hello, \\nworld!");
    }

    #[test]
    fn test_multiple_sources() {
        let mut builder = CowStringBuilder::new(0);
        builder.append_source("Hello");
        builder.append_source(", world!");
        let result = builder.finish();
        // Multiple sources force allocation
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_empty() {
        let builder = CowStringBuilder::<'static>::new(0);
        let result = builder.finish();
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }
}

