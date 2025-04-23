use crate::{
    executable_definition::{ExecutableEnumBuilder, ExecutableStructBuilder, ExecutableType},
    CodeGenerator,
};

pub(crate) struct ExecutableTypeBuilder;

impl ExecutableTypeBuilder {
    pub(crate) fn build<'a, C: CodeGenerator>(
        executable_type: &'a ExecutableType<'a>,
        code_generator: &'a C,
    ) -> Vec<syn::Item> {
        match executable_type {
            ExecutableType::Struct(es) => ExecutableStructBuilder::build(es, code_generator),
            ExecutableType::Enum(ee) => ExecutableEnumBuilder::build(ee, code_generator),
            ExecutableType::FragmentDefinitionReference { .. }
            | ExecutableType::Leaf { .. }
            | ExecutableType::BuiltinScalar { .. } => Vec::new(),
        }
    }
}
