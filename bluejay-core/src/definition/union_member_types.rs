use crate::definition::{SchemaDefinition, UnionMemberType};
use crate::AsIter;

pub trait UnionMemberTypes:
    AsIter<Item = <Self::SchemaDefinition as SchemaDefinition>::UnionMemberType>
{
    type SchemaDefinition: SchemaDefinition;

    fn contains_type(&self, name: &str) -> bool {
        self.iter().any(|t| t.name() == name)
    }

    fn get(
        &self,
        name: &str,
    ) -> Option<&<Self::SchemaDefinition as SchemaDefinition>::UnionMemberType> {
        self.iter().find(|t| t.name() == name)
    }
}
