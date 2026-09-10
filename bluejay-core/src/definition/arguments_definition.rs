use crate::definition::{InputValueDefinition, SchemaDefinition};
use crate::AsIter;

pub trait ArgumentsDefinition:
    AsIter<Item = <Self::SchemaDefinition as SchemaDefinition>::InputValueDefinition>
{
    type SchemaDefinition: SchemaDefinition;

    fn get(
        &self,
        name: &str,
    ) -> Option<&<Self::SchemaDefinition as SchemaDefinition>::InputValueDefinition> {
        self.iter().find(|fd| fd.name() == name)
    }
}
