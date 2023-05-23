mod all_variable_usages_allowed;
mod all_variable_uses_defined;
mod all_variables_used;
mod argument_names;
mod argument_uniqueness;
mod directives_are_defined;
mod directives_are_in_valid_locations;
mod directives_are_unique_per_location;
mod field_selection_merging;
mod field_selections;
mod fragment_name_uniqueness;
mod fragment_spread_is_possible;
mod fragment_spread_target_defined;
mod fragment_spread_type_exists;
mod fragment_spreads_must_not_form_cycles;
mod fragments_must_be_used;
mod fragments_on_composite_types;
mod leaf_field_selections;
mod lone_anonymous_operation;
mod named_operation_name_uniqueness;
mod operation_type_is_defined;
mod required_arguments;
mod subscription_operation_single_root_field;
mod value_is_valid;
mod variable_uniqueness;
mod variables_are_input_types;

pub use all_variable_usages_allowed::AllVariableUsagesAllowed;
pub use all_variable_uses_defined::AllVariableUsesDefined;
pub use all_variables_used::AllVariablesUsed;
pub use argument_names::ArgumentNames;
pub use argument_uniqueness::ArgumentUniqueness;
pub use directives_are_defined::DirectivesAreDefined;
pub use directives_are_in_valid_locations::DirectivesAreInValidLocations;
pub use directives_are_unique_per_location::DirectivesAreUniquePerLocation;
pub use field_selection_merging::FieldSelectionMerging;
pub use field_selections::FieldSelections;
pub use fragment_name_uniqueness::FragmentNameUniqueness;
pub use fragment_spread_is_possible::FragmentSpreadIsPossible;
pub use fragment_spread_target_defined::FragmentSpreadTargetDefined;
pub use fragment_spread_type_exists::FragmentSpreadTypeExists;
pub use fragment_spreads_must_not_form_cycles::FragmentSpreadsMustNotFormCycles;
pub use fragments_must_be_used::FragmentsMustBeUsed;
pub use fragments_on_composite_types::FragmentsOnCompositeTypes;
pub use leaf_field_selections::LeafFieldSelections;
pub use lone_anonymous_operation::LoneAnonymousOperation;
pub use named_operation_name_uniqueness::NamedOperationNameUniqueness;
pub use operation_type_is_defined::OperationTypeIsDefined;
pub use required_arguments::RequiredArguments;
pub use subscription_operation_single_root_field::SubscriptionOperationSingleRootField;
pub use value_is_valid::ValueIsValid;
pub use variable_uniqueness::VariableUniqueness;
pub use variables_are_input_types::VariablesAreInputTypes;

/// Combines multiple rules into a single rule.
/// Args:
/// 1. Name of the resulting struct
/// 2. Name of the error type returned by the rule.
///    Must accept generic lifetime, executable document, and schema definition. e.g. pass Error for `Error<'a, E, S>`.
/// 3. Rules, a comma-separated list of types accepting generic lifetime, executable document, and schema definition.
///    Must implement Rule<'a, E, S>. Must be wrapped in square brackets. e.g. `[FirstRule, SecondRule]`.
///    The `Error` type of each rule must be convertable to the error type of the new rule via `Into::into`.
#[macro_export]
macro_rules! combine_executable_rules {
    ( $name:ty, $err:ty, [$( $rule:ty ),* $(,)?] $(,)? ) => {
        paste::paste! {
            pub struct $name<'a, E: bluejay_core::executable::ExecutableDocument, S: bluejay_core::definition::SchemaDefinition> {
                $([<$rule:snake>]: $rule<'a, E, S>,)*
            }

            impl<'a, E: bluejay_core::executable::ExecutableDocument + 'a, S: bluejay_core::definition::SchemaDefinition + 'a> $crate::executable::Rule<'a, E, S> for $name<'a, E, S> {
                type Error = $err<'a, E, S>;

                fn new(executable_document: &'a E, schema_definition: &'a S, cache: &'a $crate::executable::Cache<'a, E, S>) -> Self {
                    Self {
                        $([<$rule:snake>]: $rule::new(executable_document, schema_definition, cache),)*
                    }
                }
            }

            impl<'a, E: bluejay_core::executable::ExecutableDocument + 'a, S: bluejay_core::definition::SchemaDefinition + 'a> IntoIterator for $name<'a, E, S> {
                type Item = $err<'a, E, S>;
                type IntoIter = $crate::chain_types!($(std::iter::Map<<$rule<'a, E, S> as IntoIterator>::IntoIter, fn(<$rule<'a, E, S> as $crate::executable::Rule<'a, E, S>>::Error) -> $err<'a, E, S>>),*);

                fn into_iter(self) -> Self::IntoIter {
                    $crate::chain_iters!($(self.[<$rule:snake>].into_iter().map(Into::into as fn(<$rule<'a, E, S> as $crate::executable::Rule<'a, E, S>>::Error) -> $err<'a, E, S>)),*)
                }
            }

            impl<'a, E: bluejay_core::executable::ExecutableDocument, S: bluejay_core::definition::SchemaDefinition> $crate::executable::Visitor<'a, E, S> for $name<'a, E, S> {
                fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
                    $(self.[<$rule:snake>].visit_operation_definition(operation_definition);)*
                }

                fn visit_selection_set(
                    &mut self,
                    selection_set: &'a E::SelectionSet,
                    r#type: bluejay_core::definition::TypeDefinitionReference<'a, S::TypeDefinition>,
                ) {
                    $(self.[<$rule:snake>].visit_selection_set(selection_set, r#type);)*
                }

                fn visit_field(&mut self, field: &'a E::Field, field_definition: &'a S::FieldDefinition, path: &$crate::executable::Path<'a, E>) {
                    $(self.[<$rule:snake>].visit_field(field, field_definition, path);)*
                }

                fn visit_const_directive(&mut self, directive: &'a E::Directive<true>, location: bluejay_core::definition::DirectiveLocation) {
                    $(self.[<$rule:snake>].visit_const_directive(directive, location);)*
                }

                fn visit_variable_directive(&mut self, directive: &'a E::Directive<false>, location: bluejay_core::definition::DirectiveLocation) {
                    $(self.[<$rule:snake>].visit_variable_directive(directive, location);)*
                }

                fn visit_const_directives(
                    &mut self,
                    directives: &'a E::Directives<true>,
                    location: bluejay_core::definition::DirectiveLocation,
                ) {
                    $(self.[<$rule:snake>].visit_const_directives(directives, location);)*
                }

                fn visit_variable_directives(
                    &mut self,
                    directives: &'a E::Directives<false>,
                    location: bluejay_core::definition::DirectiveLocation,
                ) {
                    $(self.[<$rule:snake>].visit_variable_directives(directives, location);)*
                }

                fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
                    $(self.[<$rule:snake>].visit_fragment_definition(fragment_definition);)*
                }

                fn visit_inline_fragment(
                    &mut self,
                    inline_fragment: &'a E::InlineFragment,
                    scoped_type: bluejay_core::definition::TypeDefinitionReference<'a, S::TypeDefinition>,
                ) {
                    $(self.[<$rule:snake>].visit_inline_fragment(inline_fragment, scoped_type);)*
                }

                fn visit_fragment_spread(
                    &mut self,
                    fragment_spread: &'a E::FragmentSpread,
                    scoped_type: bluejay_core::definition::TypeDefinitionReference<'a, S::TypeDefinition>,
                    path: &$crate::executable::Path<'a, E>,
                ) {
                    $(self.[<$rule:snake>].visit_fragment_spread(fragment_spread, scoped_type, path);)*
                }

                fn visit_const_argument(
                    &mut self,
                    argument: &'a E::Argument<true>,
                    input_value_definition: &'a S::InputValueDefinition,
                ) {
                    $(self.[<$rule:snake>].visit_const_argument(argument, input_value_definition);)*
                }

                fn visit_variable_argument(
                    &mut self,
                    argument: &'a E::Argument<false>,
                    input_value_definition: &'a S::InputValueDefinition,
                    path: &$crate::executable::Path<'a, E>,
                ) {
                    $(self.[<$rule:snake>].visit_variable_argument(argument, input_value_definition, path);)*
                }

                fn visit_variable_definition(&mut self, variable_definition: &'a E::VariableDefinition) {
                    $(self.[<$rule:snake>].visit_variable_definition(variable_definition);)*
                }

                fn visit_variable_definitions(&mut self, variable_definitions: &'a E::VariableDefinitions) {
                    $(self.[<$rule:snake>].visit_variable_definitions(variable_definitions);)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! chain_types {
    ( $first:ty, $( $rest:ty ),+ $(,)? ) => {
        std::iter::Chain<$crate::chain_types!($($rest),+), $first>
    };
    ( $t:ty ) => { $t };
}

#[macro_export]
macro_rules! chain_iters {
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        $crate::chain_iters!($($rest),+).chain($first)
    };
    ( $iter:expr ) => { $iter };
}

combine_executable_rules!(
    BuiltinRules,
    crate::executable::Error,
    [
        NamedOperationNameUniqueness,
        LoneAnonymousOperation,
        SubscriptionOperationSingleRootField,
        FieldSelections,
        FieldSelectionMerging,
        OperationTypeIsDefined,
        LeafFieldSelections,
        ArgumentNames,
        ArgumentUniqueness,
        RequiredArguments,
        FragmentNameUniqueness,
        FragmentSpreadTypeExists,
        FragmentsOnCompositeTypes,
        FragmentsMustBeUsed,
        FragmentSpreadTargetDefined,
        FragmentSpreadsMustNotFormCycles,
        FragmentSpreadIsPossible,
        ValueIsValid,
        DirectivesAreDefined,
        DirectivesAreInValidLocations,
        DirectivesAreUniquePerLocation,
        VariableUniqueness,
        VariablesAreInputTypes,
        AllVariableUsesDefined,
        AllVariablesUsed,
        AllVariableUsagesAllowed,
    ],
);
