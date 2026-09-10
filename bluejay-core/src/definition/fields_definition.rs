use crate::definition::{FieldDefinition, SchemaDefinition};
use crate::AsIter;

pub trait FieldsDefinition:
    AsIter<Item = <Self::SchemaDefinition as SchemaDefinition>::FieldDefinition>
{
    type SchemaDefinition: SchemaDefinition;

    fn contains_field(&self, name: &str) -> bool {
        self.iter().any(|fd| fd.name() == name)
    }

    fn get(
        &self,
        name: &str,
    ) -> Option<&<Self::SchemaDefinition as SchemaDefinition>::FieldDefinition> {
        self.iter().find(|fd| fd.name() == name)
    }
}
