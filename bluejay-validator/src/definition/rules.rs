mod enum_value_definition_uniqueness;
mod input_field_definition_uniqueness;
mod input_object_circular_references;

pub use enum_value_definition_uniqueness::EnumValueDefinitionUniqueness;
pub use input_field_definition_uniqueness::InputFieldDefinitionUniqueness;
pub use input_object_circular_references::InputObjectCircularReferences;

pub type BuiltinRules<'a, S> = (
    EnumValueDefinitionUniqueness<'a, S>,
    InputFieldDefinitionUniqueness<'a, S>,
    InputObjectCircularReferences<'a, S>,
);
