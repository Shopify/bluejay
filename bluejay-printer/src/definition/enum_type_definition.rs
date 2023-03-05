use crate::{
    directive::DisplayDirectives, string_value::DisplayStringValue, write_indent, INDENTATION_SIZE,
};
use bluejay_core::{
    definition::{EnumTypeDefinition, EnumValueDefinition},
    AsIter,
};
use std::fmt::{Error, Write};

pub(crate) struct DisplayEnumTypeDefinition;

impl DisplayEnumTypeDefinition {
    pub(crate) fn fmt<T: EnumTypeDefinition, W: Write>(
        enum_type_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = enum_type_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "enum {} ", enum_type_definition.name())?;

        if let Some(directives) = enum_type_definition.directives() {
            if !directives.is_empty() {
                DisplayDirectives::fmt(directives, f)?;
                write!(f, " ")?;
            }
        }

        writeln!(f, "{{")?;

        enum_type_definition
            .enum_value_definitions()
            .iter()
            .enumerate()
            .try_for_each(|(idx, evd)| {
                if idx != 0 {
                    writeln!(f)?;
                }

                if let Some(description) = evd.description() {
                    DisplayStringValue::fmt_block(description, f, INDENTATION_SIZE)?;
                }

                write_indent(f, INDENTATION_SIZE)?;
                write!(f, "{}", evd.name())?;

                if let Some(directives) = evd.directives() {
                    write!(f, " ")?;
                    DisplayDirectives::fmt(directives, f)?;
                }

                writeln!(f)
            })?;

        writeln!(f, "}}")?;

        writeln!(f)
    }
}
