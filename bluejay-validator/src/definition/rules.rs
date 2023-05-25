mod enum_value_definition_uniqueness;
mod input_field_definition_uniqueness;

pub use enum_value_definition_uniqueness::EnumValueDefinitionUniqueness;
pub use input_field_definition_uniqueness::InputFieldDefinitionUniqueness;

#[macro_export]
macro_rules! combine_definition_rules {
    ( $name:ty, $err:ty, [$( $rule:ty ),* $(,)?] $(,)? ) => {
        paste::paste! {
            pub struct $name<'a, S: bluejay_core::definition::SchemaDefinition> {
                $([<$rule:snake>]: $rule<'a, S>,)*
            }

            impl<'a, S: bluejay_core::definition::SchemaDefinition + 'a> $crate::definition::Rule<'a, S> for $name<'a, S> {
                type Error = $err<'a, S>;

                fn new(schema_definition: &'a S) -> Self {
                    Self {
                        $([<$rule:snake>]: $rule::new(schema_definition),)*
                    }
                }
            }

            impl<'a, S: bluejay_core::definition::SchemaDefinition + 'a> IntoIterator for $name<'a, S> {
                type Item = $err<'a, S>;
                type IntoIter = $crate::chain_types!($(std::iter::Map<<$rule<'a, S> as IntoIterator>::IntoIter, fn(<$rule<'a, S> as $crate::definition::Rule<'a, S>>::Error) -> $err<'a, S>>),*);

                fn into_iter(self) -> Self::IntoIter {
                    $crate::chain_iters!($(self.[<$rule:snake>].into_iter().map(Into::into as fn(<$rule<'a, S> as $crate::definition::Rule<'a, S>>::Error) -> $err<'a, S>)),*)
                }
            }

            impl<'a, S: bluejay_core::definition::SchemaDefinition> $crate::definition::Visitor<'a, S> for $name<'a, S> {
                fn visit_input_object_type_definition(&mut self, input_object_type_definition: &'a S::InputObjectTypeDefinition) {
                    $(self.[<$rule:snake>].visit_input_object_type_definition(input_object_type_definition);)*
                }

                fn visit_enum_type_definition(&mut self, enum_type_definition: &'a S::EnumTypeDefinition) {
                    $(self.[<$rule:snake>].visit_enum_type_definition(enum_type_definition);)*
                }
            }
        }
    };
}

combine_definition_rules!(
    BuiltinRules,
    crate::definition::Error,
    [
        EnumValueDefinitionUniqueness,
        InputFieldDefinitionUniqueness,
    ],
);
