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

pub type BuiltinRules<'a, E, S> = (
    NamedOperationNameUniqueness<'a, E>,
    LoneAnonymousOperation<'a, E>,
    SubscriptionOperationSingleRootField<'a, E>,
    FieldSelections<'a, E, S>,
    FieldSelectionMerging<'a, E, S>,
    OperationTypeIsDefined<'a, E, S>,
    LeafFieldSelections<'a, E, S>,
    ArgumentNames<'a, E, S>,
    ArgumentUniqueness<'a, E, S>,
    RequiredArguments<'a, E, S>,
    FragmentNameUniqueness<'a, E>,
    FragmentSpreadTypeExists<'a, E, S>,
    FragmentsOnCompositeTypes<'a, E, S>,
    FragmentsMustBeUsed<'a, E>,
    FragmentSpreadTargetDefined<'a, E, S>,
    FragmentSpreadsMustNotFormCycles<'a, E, S>,
    FragmentSpreadIsPossible<'a, E, S>,
    ValueIsValid<'a, E, S>,
    DirectivesAreDefined<'a, E, S>,
    DirectivesAreInValidLocations<'a, E, S>,
    DirectivesAreUniquePerLocation<'a, E, S>,
    VariableUniqueness<'a, E, S>,
    VariablesAreInputTypes<'a, E, S>,
    AllVariableUsesDefined<'a, E, S>,
    AllVariablesUsed<'a, E, S>,
    AllVariableUsagesAllowed<'a, E, S>,
);
