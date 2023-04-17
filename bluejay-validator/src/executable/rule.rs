use crate::executable::{Cache, Error, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Rule<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a>:
    Visitor<'a, E, S> + IntoIterator<Item = Error<'a, E, S>> + 'a
{
    fn new(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> Self;
}
