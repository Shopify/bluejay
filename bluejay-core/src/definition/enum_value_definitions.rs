use crate::definition::SchemaDefinition;
use crate::AsIter;

pub trait EnumValueDefinitions:
    AsIter<Item = <Self::SchemaDefinition as SchemaDefinition>::EnumValueDefinition>
{
    type SchemaDefinition: SchemaDefinition;
}
