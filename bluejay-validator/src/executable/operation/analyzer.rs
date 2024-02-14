use crate::executable::operation::{VariableValues, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Analyzer<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues>:
    Visitor<'a, E, S, V> + Into<Self::Output>
{
    type Output;
}
