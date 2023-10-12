mod changes;
mod diff;
mod result;

use bluejay_core::definition::SchemaDefinition;
pub use changes::{Change, Criticality};
pub use result::ComparisonResult;

pub fn compare<'a, S: SchemaDefinition>(
    old_schema: &'a S,
    new_schema: &'a S,
) -> ComparisonResult<'a, S> {
    let schema = diff::SchemaDiff::new(old_schema, new_schema);

    ComparisonResult::new(schema.diff())
}
