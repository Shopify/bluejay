use crate::{
    directive::DirectivesPrinter, string_value::BlockStringValuePrinter, write_indent,
    INDENTATION_SIZE,
};
use bluejay_core::{
    definition::{EnumTypeDefinition, EnumValueDefinition},
    AsIter,
};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct EnumTypeDefinitionPrinter<'a, E: EnumTypeDefinition>(&'a E);

impl<'a, E: EnumTypeDefinition> EnumTypeDefinitionPrinter<'a, E> {
    pub(crate) fn new(enum_type_definition: &'a E) -> Self {
        Self(enum_type_definition)
    }
}

impl<'a, E: EnumTypeDefinition> Display for EnumTypeDefinitionPrinter<'a, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(enum_type_definition) = *self;
        if let Some(description) = enum_type_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "enum {} ", enum_type_definition.name())?;

        if let Some(directives) = enum_type_definition.directives() {
            if !directives.is_empty() {
                write!(f, "{} ", DirectivesPrinter::new(directives))?;
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
                    write!(
                        f,
                        "{}",
                        BlockStringValuePrinter::new(description, INDENTATION_SIZE)
                    )?;
                }

                write_indent(f, INDENTATION_SIZE)?;
                write!(f, "{}", evd.name())?;

                if let Some(directives) = evd.directives() {
                    if !directives.is_empty() {
                        write!(f, " {}", DirectivesPrinter::new(directives))?;
                    }
                }

                writeln!(f)
            })?;

        writeln!(f, "}}")
    }
}
