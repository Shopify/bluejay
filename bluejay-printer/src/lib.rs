pub mod format;
pub mod serializer;

pub use format::{escape_block_string, CompactFormatter, Formatter, PrettyFormatter};
pub use serializer::Serializer;

use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;
use bluejay_core::Value;

pub fn print_schema_definition<S: SchemaDefinition>(schema_definition: &S) -> String {
    let mut s = Serializer::to_string(PrettyFormatter::default());
    s.serialize_schema_definition(schema_definition).unwrap();
    s.into_inner()
}

pub fn print_executable_document<T: ExecutableDocument>(executable_document: &T) -> String {
    let mut s = Serializer::to_string(PrettyFormatter::default());
    s.serialize_executable_document(executable_document)
        .unwrap();
    s.into_inner()
}

pub fn print_value<const CONST: bool, V: Value<CONST>>(value: &V) -> String {
    let mut s = Serializer::to_string(PrettyFormatter::default());
    s.serialize_value(value).unwrap();
    s.into_inner()
}
