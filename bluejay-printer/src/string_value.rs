use crate::write_indent;
use std::fmt::{Display, Formatter, Result};

pub(crate) struct StringValuePrinter<'a>(&'a str);

impl<'a> StringValuePrinter<'a> {
    pub(crate) fn new(value: &'a str) -> Self {
        Self(value)
    }
}

impl Display for StringValuePrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(value) = *self;
        write!(f, "\"")?;
        value.chars().try_for_each(|c| match c {
            '\"' | '\\' => write!(f, "\\{c}"),
            '\u{0008}' => write!(f, "\\b"),
            '\u{000C}' => write!(f, "\\f"),
            '\n' => write!(f, "\\n"),
            '\r' => write!(f, "\\r"),
            '\t' => write!(f, "\\t"),
            c if c.is_control() => write!(f, "\\u{:04X}", c as u32),
            c => write!(f, "{c}"),
        })?;
        write!(f, "\"")
    }
}

pub(crate) struct BlockStringValuePrinter<'a> {
    value: &'a str,
    indentation: usize,
}

impl<'a> BlockStringValuePrinter<'a> {
    pub(crate) fn new(value: &'a str, indentation: usize) -> Self {
        Self { value, indentation }
    }
}

/// Return true if a block string, in the format that `BlockStringValuePrinter`
/// writes, parses back to exactly `value`.
///
/// The printer writes the value on its own lines between the delimiters.
/// Block string parsing treats `\r` as a line terminator, and removes the
/// common indentation and the leading and trailing blank lines. So the value
/// must not have:
/// - `\r`, or a control character other than tab and line feed
/// - a leading or trailing blank line
/// - indentation that is common to all of its non-blank lines
fn is_printable_as_block_string(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    if value
        .chars()
        .any(|c| c.is_control() && c != '\t' && c != '\n')
    {
        return false;
    }
    let is_blank = |line: &str| line.chars().all(|c| c == ' ' || c == '\t');
    let lines: Vec<&str> = value.split('\n').collect();
    if lines.first().copied().is_some_and(is_blank) || lines.last().copied().is_some_and(is_blank) {
        return false;
    }
    lines
        .iter()
        .any(|line| !is_blank(line) && !line.starts_with([' ', '\t']))
}

impl Display for BlockStringValuePrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { value, indentation } = *self;
        write_indent(f, indentation)?;

        if !is_printable_as_block_string(value) {
            return writeln!(f, "{}", StringValuePrinter::new(value));
        }

        writeln!(f, "\"\"\"")?;

        let escaped = value.replace("\"\"\"", "\\\"\"\"");

        escaped.lines().try_for_each(|line| {
            write_indent(f, indentation)?;
            writeln!(f, "{line}")
        })?;

        write_indent(f, indentation)?;
        writeln!(f, "\"\"\"")
    }
}

#[cfg(test)]
mod tests {
    use super::BlockStringValuePrinter;

    fn assert_prints_block(expected_output: &str, input: &str, indentation: usize) {
        let output = BlockStringValuePrinter::new(input, indentation).to_string();
        assert_eq!(expected_output, output);
    }

    #[test]
    fn test_block() {
        assert_prints_block("\"\"\"\n\"\"\"\n", "", 0);
        assert_prints_block("    \"\"\"\n    \"\"\"\n", "", 4);
        assert_prints_block(
            "\"\"\"\nThis\nis\na\nmultiline\nstring\n\"\"\"\n",
            "This\nis\na\nmultiline\nstring",
            0,
        );
        assert_prints_block("\"\"\"\n\\\"\"\"\n\"\"\"\n", "\"\"\"", 0);
        assert_prints_block("  \"\"\"\n  a\n    b\n  \"\"\"\n", "a\n  b", 2);
    }

    #[test]
    fn test_block_falls_back_to_string() {
        assert_prints_block("\"\\nleading newline\"\n", "\nleading newline", 0);
        assert_prints_block("\"trailing newline\\n\"\n", "trailing newline\n", 0);
        assert_prints_block("  \"  common indent\"\n", "  common indent", 2);
        assert_prints_block("\"a\\r\\nb\"\n", "a\r\nb", 0);
        assert_prints_block("\"a\\u0001b\"\n", "a\u{1}b", 0);
    }

    #[test]
    fn test_block_round_trips() {
        use bluejay_core::definition::SchemaDefinition as _;
        use bluejay_parser::ast::{
            definition::{DefinitionDocument, SchemaDefinition},
            Parse,
        };

        let values = [
            "",
            "a",
            "a\n  b",
            "a\n\n\nb",
            "a\n   \nb",
            "\nleading newline",
            "trailing newline\n",
            "  leading spaces",
            "  a\n  b",
            "\t",
            "a\r\nb",
            "a\rb",
            "a\u{1}b\u{7f}c\u{85}",
            "\"\"\"",
            "\\\"\"\"",
            "ends with a quote\"",
            "ends with a backslash\\",
        ];
        for value in values {
            let source = format!(
                "{}scalar S\n\ntype Query {{\n  s: S\n}}\n",
                BlockStringValuePrinter::new(value, 0)
            );
            let document: DefinitionDocument = DefinitionDocument::parse(&source)
                .result
                .unwrap_or_else(|errors| panic!("{source:?} does not parse: {errors:?}"));
            let schema = SchemaDefinition::try_from(&document).unwrap();
            let description = schema
                .get_type_definition("S")
                .and_then(|t| t.description());
            assert_eq!(Some(value), description, "{source:?}");
        }
    }
}
