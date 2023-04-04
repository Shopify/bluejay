mod argument_names;
mod argument_uniqueness;
mod field_selection_merging;
mod field_selections;
mod fragment_name_uniqueness;
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

pub use argument_names::ArgumentNames;
pub use argument_uniqueness::ArgumentUniqueness;
pub use field_selection_merging::FieldSelectionMerging;
pub use field_selections::FieldSelections;
pub use fragment_name_uniqueness::FragmentNameUniqueness;
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

use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::{
    DirectiveLocation, SchemaDefinition, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};
use paste::paste;
use std::iter::Chain;

macro_rules! define_rules {
    ( $( $rule:ty ),* $(,)? ) => {
        paste! {
            pub struct Rules<'a, E: ExecutableDocument, S: SchemaDefinition> {
                $([<$rule:snake>]: $rule<'a, E, S>,)*
            }

            impl<'a, E: ExecutableDocument, S: SchemaDefinition> Rule<'a, E, S> for Rules<'a, E, S> {
                fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
                    Self {
                        $([<$rule:snake>]: $rule::new(executable_document, schema_definition),)*
                    }
                }
            }

            impl<'a, E: ExecutableDocument, S: SchemaDefinition> IntoIterator for Rules<'a, E, S> {
                type Item = Error<'a, E, S>;
                type IntoIter = chain_types!($(<$rule<'a, E, S> as IntoIterator>::IntoIter),*);

                fn into_iter(self) -> Self::IntoIter {
                    chain_iters!($(self.[<$rule:snake>].into_iter()),*)
                }
            }

            impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S> for Rules<'a, E, S> {
                fn visit_operation_definition(&mut self, operation_definition: &'a OperationDefinitionFromExecutableDocument<E>) {
                    $(self.[<$rule:snake>].visit_operation_definition(operation_definition);)*
                }

                fn visit_selection_set(
                    &mut self,
                    selection_set: &'a E::SelectionSet,
                    r#type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
                ) {
                    $(self.[<$rule:snake>].visit_selection_set(selection_set, r#type);)*
                }

                fn visit_field(&mut self, field: &'a E::Field, field_definition: &'a S::FieldDefinition) {
                    $(self.[<$rule:snake>].visit_field(field, field_definition);)*
                }

                fn visit_const_directive(&mut self, directive: &'a E::Directive<true>, location: DirectiveLocation) {
                    $(self.[<$rule:snake>].visit_const_directive(directive, location);)*
                }

                fn visit_variable_directive(&mut self, directive: &'a E::Directive<false>, location: DirectiveLocation) {
                    $(self.[<$rule:snake>].visit_variable_directive(directive, location);)*
                }

                fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
                    $(self.[<$rule:snake>].visit_fragment_definition(fragment_definition);)*
                }

                fn visit_inline_fragment(&mut self, inline_fragment: &'a E::InlineFragment) {
                    $(self.[<$rule:snake>].visit_inline_fragment(inline_fragment);)*
                }

                fn visit_fragment_spread(&mut self, fragment_spread: &'a E::FragmentSpread) {
                    $(self.[<$rule:snake>].visit_fragment_spread(fragment_spread);)*
                }
            }
        }
    };
}

macro_rules! chain_types {
    ( $first:ty, $( $rest:ty ),+ $(,)? ) => {
        Chain<chain_types!($($rest),+), $first>
    };
    ( $t:ty ) => { $t };
}

macro_rules! chain_iters {
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        chain_iters!($($rest),+).chain($first)
    };
    ( $iter:expr ) => { $iter };
}

define_rules!(
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
);
