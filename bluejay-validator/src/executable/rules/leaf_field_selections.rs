use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, Field};

pub struct LeafFieldSelections<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for LeafFieldSelections<'a, E, S>
{
    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        r#type: &'a S::OutputTypeReference,
    ) {
        if r#type.as_ref().base().is_scalar_or_enum() {
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
    fn new(_: &'a E, _: &'a S) -> Self {
        Self { errors: Vec::new() }
    }
}
