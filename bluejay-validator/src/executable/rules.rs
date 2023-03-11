mod field_selection_merging;
mod field_selections;
mod leaf_field_selections;
mod lone_anonymous_operation;
mod named_operation_name_uniqueness;
mod operation_type_is_defined;
mod subscription_operation_single_root_field;

use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use bluejay_core::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};
use field_selection_merging::FieldSelectionMerging;
use field_selections::FieldSelections;
use leaf_field_selections::LeafFieldSelections;
use lone_anonymous_operation::LoneAnonymousOperation;
use named_operation_name_uniqueness::NamedOperationNameUniqueness;
use operation_type_is_defined::OperationTypeIsDefined;
use paste::paste;
use std::iter::Chain;
use subscription_operation_single_root_field::SubscriptionOperationSingleRootField;

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
                fn visit_operation(&mut self, operation_definition: &'a OperationDefinitionFromExecutableDocument<E>) {
                    $(self.[<$rule:snake>].visit_operation(operation_definition);)*
                }

                fn visit_selection_set(&mut self, selection_set: &'a E::SelectionSet, r#type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>) {
                    $(self.[<$rule:snake>].visit_selection_set(selection_set, r#type);)*
                }

                fn visit_field(&mut self, field: &'a E::Field, r#type: &'a S::OutputTypeReference) {
                    $(self.[<$rule:snake>].visit_field(field, r#type);)*
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
);
