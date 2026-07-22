use std::fmt;

/// Escapes triple-quote sequences in a block string by replacing `"""` with `\"""`.
pub fn escape_block_string(s: &str) -> String {
    s.replace("\"\"\"", "\\\"\"\"")
}

/// Controls the whitespace and formatting style used by [`crate::Serializer`] when
/// writing GraphQL output.
///
/// Implement this trait to define custom formatting strategies beyond the
/// provided [`PrettyFormatter`] and [`CompactFormatter`].
///
/// The `&mut self` receiver on every method allows formatters to maintain state
/// (e.g. tracking the current column for line-wrapping heuristics).
///
/// All `depth` parameters represent a logical nesting level (0 = top-level,
/// 1 = one level in, etc.). The formatter decides how to render that
/// (e.g. multiply by an indent width, or ignore it entirely).
pub trait Formatter {
    /// Writes indentation for the given nesting `depth`.
    /// Called before block members, field definitions, enum values, and
    /// similar indented constructs.
    fn write_indent<W: fmt::Write>(&mut self, w: &mut W, depth: usize) -> fmt::Result;

    /// Opens a brace-delimited block (`{`). Called at the start of selection
    /// sets, field definition lists, enum bodies, and similar constructs.
    /// `depth` is the nesting level of the block being opened.
    fn begin_block<W: fmt::Write>(&mut self, w: &mut W, depth: usize) -> fmt::Result;

    /// Closes a brace-delimited block (`}`). `depth` is the nesting level
    /// of the block being closed (i.e. the same depth that was current when
    /// [`begin_block`](Formatter::begin_block) was called).
    fn end_block<W: fmt::Write>(&mut self, w: &mut W, depth: usize) -> fmt::Result;

    /// Terminates a line after a statement, field definition, or enum value.
    fn write_line_ending<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Separates consecutive items inside a block (e.g. between two field
    /// definitions or two enum values that each have their own line).
    fn write_block_item_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Separates items in an inline comma-delimited list such as arguments,
    /// variable definitions, or list-value elements.
    fn write_list_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Separates top-level definitions (types, directives, the schema block).
    fn write_definition_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Writes a GraphQL block-string description (`""" ... """`).
    /// `depth` is the current nesting level. Implementations may choose to
    /// omit descriptions entirely (as [`CompactFormatter`] does).
    fn write_description<W: fmt::Write>(
        &mut self,
        w: &mut W,
        desc: &str,
        depth: usize,
    ) -> fmt::Result;

    /// Opens an object-value literal (e.g. `{ ` in pretty mode, `{` in compact).
    fn begin_value_object<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Closes an object-value literal (e.g. ` }` in pretty mode, `}` in compact).
    fn end_value_object<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Writes the space (or other whitespace) that appears before an opening
    /// brace-delimited block. Called immediately before [`begin_block`](Formatter::begin_block).
    fn write_space_before_block<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Writes the separator between a key and its value (e.g. `": "` in pretty
    /// mode, `":"` in compact). Used for type annotations, argument key-value
    /// pairs, object value entries, field aliases, and variable definitions.
    fn write_key_value_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Writes the separator between union members or directive locations
    /// (e.g. `" | "` in pretty mode, `"|"` in compact).
    fn write_union_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;

    /// Writes the `=` separator used in default values and union type definitions
    /// (e.g. `" = "` in pretty mode, `"="` in compact).
    fn write_equals<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result;
}

#[derive(Debug, Clone)]
pub struct PrettyFormatter {
    indent_size: usize,
}

impl PrettyFormatter {
    pub fn new(indent_size: usize) -> Self {
        Self { indent_size }
    }
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        Self { indent_size: 2 }
    }
}

impl Formatter for PrettyFormatter {
    fn write_indent<W: fmt::Write>(&mut self, w: &mut W, depth: usize) -> fmt::Result {
        let chars = depth * self.indent_size;
        write!(w, "{: >1$}", "", chars)
    }

    fn begin_block<W: fmt::Write>(&mut self, w: &mut W, _depth: usize) -> fmt::Result {
        writeln!(w, "{{")
    }

    fn end_block<W: fmt::Write>(&mut self, w: &mut W, depth: usize) -> fmt::Result {
        self.write_indent(w, depth)?;
        write!(w, "}}")
    }

    fn write_line_ending<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        writeln!(w)
    }

    fn write_block_item_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        writeln!(w)
    }

    fn write_list_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, ", ")
    }

    fn write_definition_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        writeln!(w)
    }

    fn write_description<W: fmt::Write>(
        &mut self,
        w: &mut W,
        desc: &str,
        depth: usize,
    ) -> fmt::Result {
        let chars = depth * self.indent_size;
        write!(w, "{: >1$}", "", chars)?;
        writeln!(w, "\"\"\"")?;

        let escaped = escape_block_string(desc);

        for line in escaped.lines() {
            write!(w, "{: >1$}", "", chars)?;
            writeln!(w, "{line}")?;
        }

        write!(w, "{: >1$}", "", chars)?;
        writeln!(w, "\"\"\"")
    }

    fn begin_value_object<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, "{{ ")
    }

    fn end_value_object<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, " }}")
    }

    fn write_space_before_block<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, " ")
    }

    fn write_key_value_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, ": ")
    }

    fn write_union_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, " | ")
    }

    fn write_equals<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, " = ")
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn write_indent<W: fmt::Write>(&mut self, _w: &mut W, _depth: usize) -> fmt::Result {
        Ok(())
    }

    fn begin_block<W: fmt::Write>(&mut self, w: &mut W, _depth: usize) -> fmt::Result {
        write!(w, "{{")
    }

    fn end_block<W: fmt::Write>(&mut self, w: &mut W, _depth: usize) -> fmt::Result {
        write!(w, "}}")
    }

    fn write_line_ending<W: fmt::Write>(&mut self, _w: &mut W) -> fmt::Result {
        Ok(())
    }

    fn write_block_item_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, " ")
    }

    fn write_list_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, ",")
    }

    fn write_definition_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, " ")
    }

    fn write_description<W: fmt::Write>(
        &mut self,
        _w: &mut W,
        _desc: &str,
        _depth: usize,
    ) -> fmt::Result {
        Ok(())
    }

    fn begin_value_object<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, "{{")
    }

    fn end_value_object<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, "}}")
    }

    fn write_space_before_block<W: fmt::Write>(&mut self, _w: &mut W) -> fmt::Result {
        Ok(())
    }

    fn write_key_value_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, ":")
    }

    fn write_union_separator<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, "|")
    }

    fn write_equals<W: fmt::Write>(&mut self, w: &mut W) -> fmt::Result {
        write!(w, "=")
    }
}
