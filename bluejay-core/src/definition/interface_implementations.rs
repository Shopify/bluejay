use crate::definition::SchemaDefinition;
use crate::AsIter;

pub trait InterfaceImplementations:
    AsIter<Item = <Self::SchemaDefinition as SchemaDefinition>::InterfaceImplementation>
{
    type SchemaDefinition: SchemaDefinition;
}
