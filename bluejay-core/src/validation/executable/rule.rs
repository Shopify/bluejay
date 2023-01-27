use crate::validation::executable::{Visitor, Error};
use crate::executable::ExecutableDocument;
use crate::definition::SchemaDefinition;

pub trait Rule<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>>: Visitor<'a, E, S> + IntoIterator<Item = Error<'a, E, S>> + 'a {
    fn new(executable_document: &'a E, schema_definition: &'a S) -> Self;
}
