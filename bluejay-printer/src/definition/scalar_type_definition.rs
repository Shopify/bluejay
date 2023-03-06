use crate::{directive::DisplayDirectives, string_value::DisplayStringValue};
use bluejay_core::{definition::ScalarTypeDefinition, AsIter};
use std::fmt::{Error, Write};

pub(crate) struct DisplayScalarTypeDefinition;

impl DisplayScalarTypeDefinition {
    pub(crate) fn fmt<T: ScalarTypeDefinition, W: Write>(
        scalar_type_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = scalar_type_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "scalar {}", scalar_type_definition.name())?;

        if let Some(directives) = scalar_type_definition.directives() {
            if !directives.is_empty() {
                write!(f, " ")?;
                DisplayDirectives::fmt(directives, f)?;
            }
        }

        writeln!(f)
    }
}
