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

impl Display for BlockStringValuePrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { value, indentation } = *self;
        write_indent(f, indentation)?;
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
    }
}
