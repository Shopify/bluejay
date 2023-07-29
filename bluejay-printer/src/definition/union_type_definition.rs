use crate::{directive::DirectivesPrinter, string_value::BlockStringValuePrinter};
use bluejay_core::{
    definition::{ObjectTypeDefinition, UnionMemberType, UnionTypeDefinition},
    AsIter,
};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct UnionTypeDefinitionPrinter<'a, U: UnionTypeDefinition>(&'a U);

impl<'a, U: UnionTypeDefinition> UnionTypeDefinitionPrinter<'a, U> {
    pub(crate) fn new(union_type_definition: &'a U) -> Self {
        Self(union_type_definition)
    }
}

impl<'a, U: UnionTypeDefinition> Display for UnionTypeDefinitionPrinter<'a, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(union_type_definition) = *self;
        if let Some(description) = union_type_definition.description() {
            write!(f, "{}", BlockStringValuePrinter::new(description, 0))?;
        }

        write!(f, "union {}", union_type_definition.name())?;

        if let Some(directives) = union_type_definition.directives() {
            if !directives.is_empty() {
                write!(f, " {}", DirectivesPrinter::new(directives))?;
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
