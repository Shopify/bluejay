use crate::{
    executable_definition::{ExecutableEnumBuilder, ExecutableStructBuilder, ExecutableType},
    Config,
};
use bluejay_core::definition::SchemaDefinition;

pub(crate) struct ExecutableTypeBuilder;

impl ExecutableTypeBuilder {
    pub(crate) fn build<'a, S: SchemaDefinition>(
        executable_type: &'a ExecutableType<'a>,
        config: &'a Config<'a, S>,
        depth: usize,
    ) -> Vec<syn::Item> {
        match executable_type {
            ExecutableType::Struct(es) => ExecutableStructBuilder::build(es, config, depth),
            ExecutableType::Enum(ee) => ExecutableEnumBuilder::build(ee, config, depth),
            ExecutableType::FragmentDefinitionReference { .. }
            | ExecutableType::Leaf { .. }
            | ExecutableType::BuiltinScalar { .. } => Vec::new(),
        }
    }
}
