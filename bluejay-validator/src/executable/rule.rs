use crate::executable::Visitor;
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Rule<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a>:
    Visitor<'a, E, S> + IntoIterator<Item = Self::Error>
{
    type Error;
}
