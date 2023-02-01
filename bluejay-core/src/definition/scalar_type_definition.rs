pub trait ScalarTypeDefinition {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
}
