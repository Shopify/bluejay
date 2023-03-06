use crate::{directive::DisplayDirectives, string_value::DisplayStringValue};
use bluejay_core::{
    definition::{ObjectTypeDefinition, UnionMemberType, UnionTypeDefinition},
    AsIter,
};
use std::fmt::{Error, Write};

pub(crate) struct DisplayUnionTypeDefinition;

impl DisplayUnionTypeDefinition {
    pub(crate) fn fmt<T: UnionTypeDefinition, W: Write>(
        union_type_definition: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if let Some(description) = union_type_definition.description() {
            DisplayStringValue::fmt_block(description, f, 0)?;
        }

        write!(f, "union {}", union_type_definition.name())?;

        if let Some(directives) = union_type_definition.directives() {
            if !directives.is_empty() {
                DisplayDirectives::fmt(directives, f)?;
            }
        }

        write!(f, " = ")?;

        union_type_definition
            .union_member_types()
            .iter()
            .enumerate()
            .try_for_each(|(idx, union_member)| {
                if idx != 0 {
                    write!(f, " | ")?;
                }
                write!(f, "{}", union_member.member_type().name())
            })?;

        writeln!(f)
    }
}
