use crate::executable::{Cache, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Rule<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a>:
    Visitor<'a, E, S> + IntoIterator<Item = Self::Error>
{
    type Error;

    fn new(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> Self;
}
