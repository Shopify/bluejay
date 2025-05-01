use bluejay_core::{
    definition::{prelude::*, BaseOutputTypeReference, SchemaDefinition},
    executable::{ExecutableDocument, Field, Selection, SelectionReference},
};
use bluejay_validator::executable::{
    document::{Analyzer, Path, Visitor},
    Cache,
};
use std::collections::HashSet;

pub(crate) struct PathsWithCustomScalarType<'a, S: SchemaDefinition> {
    schema_definition: &'a S,
    paths: HashSet<Vec<String>>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for PathsWithCustomScalarType<'a, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            paths: HashSet::new(),
        }
    }

    fn visit_field(
        &mut self,
        _field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a <S as SchemaDefinition>::FieldDefinition,
        path: &Path<'a, E>,
    ) {
        if matches!(
            field_definition.r#type().base(self.schema_definition),
            BaseOutputTypeReference::CustomScalar(_)
        ) {
            if let Some(root_name) = path.root().name() {
                self.paths.insert(
                    std::iter::once(root_name.to_string())
                        .chain(path.members().iter().filter_map(
                            |selection| match selection.as_ref() {
                                SelectionReference::Field(f) => Some(f.response_name().to_string()),
                                _ => None,
                            },
                        ))
                        .collect(),
                );
            }
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Analyzer<'a, E, S>
    for PathsWithCustomScalarType<'a, S>
{
    type Output = HashSet<Vec<String>>;

    fn into_output(self) -> Self::Output {
        self.paths
    }
}
