use super::changes::Change;
use bluejay_core::definition::SchemaDefinition;

pub struct ComparisonResult<'a, S: SchemaDefinition> {
    pub changes: Vec<Change<'a, S>>,
}

impl<'a, S: SchemaDefinition> ComparisonResult<'a, S> {
    pub fn new(mut changes: Vec<Change<'a, S>>) -> Self {
        changes.sort_by_key(|b| std::cmp::Reverse(b.criticality()));

        Self { changes }
    }
}
