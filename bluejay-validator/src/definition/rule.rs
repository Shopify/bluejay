use crate::definition::Visitor;
use bluejay_core::definition::SchemaDefinition;

pub trait Rule<'a, S: SchemaDefinition>: Visitor<'a, S> + IntoIterator<Item = Self::Error> {
    type Error;

    fn new(schema_definition: &'a S) -> Self;
}
