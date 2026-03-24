use super::Token as OuterToken;
use crate::lexer::LexError;
use logos::Lexer;
use std::borrow::Cow;

pub(super) struct Token;

#[inline]
fn after_indent(line: &str, indent: usize) -> &str {
    if indent < line.len() {
        &line[indent..]
    } else {
        ""
    }
}

impl Token {
    /// Parse a block string value from the outer lexer.
    /// The opening `"""` has already been consumed.
    pub(super) fn parse<'a>(
        outer_lexer: &mut Lexer<'a, OuterToken<'a>>,
    ) -> Result<Cow<'a, str>, LexError> {
        let remainder = outer_lexer.remainder();
        let bytes = remainder.as_bytes();
        let len = bytes.len();

        // Find the closing """ (not preceded by \)
        let mut i = 0;
        let mut has_escapes = false;
        let end_offset;
        loop {
            if i + 2 >= len {
                outer_lexer.bump(len);
                return Err(LexError::UnrecognizedToken);
            }
            if bytes[i] == b'"' && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
                // Check it's not escaped
                if i > 0 && bytes[i - 1] == b'\\' {
                    has_escapes = true;
                    i += 3;
                    continue;
                }
                end_offset = i + 3;
                break;
            }
            i += 1;
        }

        let raw = &remainder[..i];
        outer_lexer.bump(end_offset);

        // Normalize newlines: split on \r\n, \r, or \n
        // Collect line start/end offsets to avoid allocating strings
        let raw_bytes = raw.as_bytes();
        let raw_len = raw_bytes.len();

        // Count lines first for pre-allocation
        let mut line_count = 1usize;
        {
            let mut j = 0;
            while j < raw_len {
                if raw_bytes[j] == b'\r' {
                    line_count += 1;
                    if j + 1 < raw_len && raw_bytes[j + 1] == b'\n' {
                        j += 2;
                    } else {
                        j += 1;
                    }
                } else if raw_bytes[j] == b'\n' {
                    line_count += 1;
                    j += 1;
                } else {
                    j += 1;
                }
            }
        }

        // Collect line ranges
        let mut lines: Vec<(usize, usize)> = Vec::with_capacity(line_count);
        {
            let mut start = 0;
            let mut j = 0;
            while j < raw_len {
                if raw_bytes[j] == b'\r' {
                    lines.push((start, j));
                    if j + 1 < raw_len && raw_bytes[j + 1] == b'\n' {
                        j += 2;
                    } else {
                        j += 1;
                    }
                    start = j;
                } else if raw_bytes[j] == b'\n' {
                    lines.push((start, j));
                    j += 1;
                    start = j;
                } else {
                    j += 1;
                }
            }
            lines.push((start, raw_len));
        }

        // Compute common indent (skip first line)
        let common_indent = lines[1..]
            .iter()
            .filter_map(|&(start, end)| {
                let line_bytes = raw[start..end].as_bytes();
                let indent = line_bytes.iter().position(|&b| b != b' ' && b != b'\t');
                indent // None means all-whitespace, filtered out
            })
            .min()
            .unwrap_or(0);

        // Find first non-blank line
        let front_offset = lines.iter().enumerate().position(|(idx, &(start, end))| {
            let indent = if idx == 0 { 0 } else { common_indent };
            let line = &raw[start..end];
            after_indent(line, indent)
                .as_bytes()
                .iter()
                .any(|&b| b != b' ' && b != b'\t')
        });

        // Find last non-blank line
        let end_offset_lines = lines.iter().rev().position(|&(start, end)| {
            let line = &raw[start..end];
            after_indent(line, common_indent)
                .as_bytes()
                .iter()
                .any(|&b| b != b' ' && b != b'\t')
        });

        if let Some((front, end_off)) = front_offset.zip(end_offset_lines) {
            let first = front;
            let last = lines.len() - end_off; // exclusive

            if !has_escapes && first == 0 && last == 1 {
                // Single line, no escapes — can return borrowed
                let (start, end) = lines[0];
                let line = &raw[start..end];
                return Ok(Cow::Borrowed(line));
            }

            // Check if we can return a borrowed slice (single content line from source, no escapes, not first line)
            if !has_escapes && first + 1 == last && first > 0 {
                let (start, end) = lines[first];
                let line = &raw[start..end];
                return Ok(Cow::Borrowed(after_indent(line, common_indent)));
            }

            // Build the result string
            let mut result = String::new();
            for (offset_idx, line_idx) in (first..last).enumerate() {
                let (start, end) = lines[line_idx];
                let indent = if line_idx == 0 { 0 } else { common_indent };
                if offset_idx != 0 {
                    result.push('\n');
                }
                let line = &raw[start..end];
                let trimmed = after_indent(line, indent);
                if has_escapes {
                    result.push_str(&trimmed.replace("\\\"\"\"", "\"\"\""));
                } else {
                    result.push_str(trimmed);
                }
            }

            Ok(Cow::Owned(result))
        } else {
            Ok(Cow::Borrowed(""))
        }
    }
}
