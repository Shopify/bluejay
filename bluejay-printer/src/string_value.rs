use crate::write_indent;
use std::fmt::{Error, Write};

pub(crate) struct DisplayStringValue;

impl DisplayStringValue {
    pub(crate) fn fmt<W: Write>(s: &str, f: &mut W) -> Result<(), Error> {
        write!(f, "\"")?;
        s.chars().try_for_each(|c| match c {
            '\"' | '\\' | '/' => write!(f, "\\{c}"),
            '\u{0008}' => write!(f, "\\b"),
            '\u{000C}' => write!(f, "\\f"),
            '\n' => write!(f, "\\n"),
            '\r' => write!(f, "\\r"),
            '\t' => write!(f, "\\t"),
            c => write!(f, "{c}"),
        })?;
        write!(f, "\"")
    }

    pub(crate) fn fmt_block<W: Write>(s: &str, f: &mut W, indentation: usize) -> Result<(), Error> {
        write_indent(f, indentation)?;
        writeln!(f, "\"\"\"")?;

        let escaped = s.replace("\"\"\"", "\\\"\"\"");

        escaped.lines().try_for_each(|line| {
            write_indent(f, indentation)?;
            writeln!(f, "{line}")
        })?;

        write_indent(f, indentation)?;
        writeln!(f, "\"\"\"")
    }

    #[cfg(test)]
    fn to_block_string(s: &str, indentation: usize) -> String {
        let mut formatted = String::new();
        Self::fmt_block(s, &mut formatted, indentation)
            .expect("fmt_block returned an error unexpectedly");
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayStringValue;

    fn assert_prints_block(expected_output: &str, input: &str, indentation: usize) {
        let output = DisplayStringValue::to_block_string(input, indentation);
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
