use crate::executable::{Cache, Error, Path, Rule, Visitor};
use bluejay_core::definition::{FieldDefinition, OutputType, SchemaDefinition};
use bluejay_core::executable::{ExecutableDocument, Field};

pub struct LeafFieldSelections<'a, E: ExecutableDocument, S: SchemaDefinition> {
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for LeafFieldSelections<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            schema_definition,
            errors: Vec::new(),
        }
    }

    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a S::FieldDefinition,
        _: &Path<'a, E>,
    ) {
        let r#type = field_definition.r#type();
        if r#type.base(self.schema_definition).is_scalar_or_enum() {
            if let Some(selection_set) = field.selection_set() {
                self.errors.push(Error::LeafFieldSelectionNotEmpty {
                    selection_set,
                    r#type,
                });
            }
        } else if field.selection_set().is_none() {
            self.errors
                .push(Error::NonLeafFieldSelectionEmpty { field, r#type });
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for LeafFieldSelections<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for LeafFieldSelections<'a, E, S>
{
    type Error = Error<'a, E, S>;
}
