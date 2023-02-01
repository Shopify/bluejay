mod field_selection_merging;
mod field_selections;
mod lone_anonymous_operation;
mod named_operation_name_uniqueness;
mod subscription_operation_single_root_field;

use crate::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use crate::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};
use crate::validation::executable::{Error, Rule, Visitor};
use field_selection_merging::FieldSelectionMerging;
use field_selections::FieldSelections;
use lone_anonymous_operation::LoneAnonymousOperation;
use named_operation_name_uniqueness::NamedOperationNameUniqueness;
use paste::paste;
use std::iter::Chain;
use subscription_operation_single_root_field::SubscriptionOperationSingleRootField;

macro_rules! define_rules {
    ( $( $rule:ty ),* $(,)? ) => {
        paste! {
            pub struct Rules<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> {
                $([<$rule:snake>]: $rule<'a, E, S>,)*
            }

            impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Rule<'a, E, S> for Rules<'a, E, S> {
                fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
                    Self {
                        $([<$rule:snake>]: $rule::new(executable_document, schema_definition),)*
                    }
                }
            }

            impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> IntoIterator for Rules<'a, E, S> {
                type Item = Error<'a, E, S>;
                type IntoIter = chain_types!($(<$rule<'a, E, S> as IntoIterator>::IntoIter),*);

                fn into_iter(self) -> Self::IntoIter {
                    chain_iters!($(self.[<$rule:snake>].into_iter()),*)
                }
            }

            impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Visitor<'a, E, S> for Rules<'a, E, S> {
                fn visit_operation(&mut self, operation_definition: &'a OperationDefinitionFromExecutableDocument<'a, E>) {
                    $(self.[<$rule:snake>].visit_operation(operation_definition);)*
                }

                fn visit_selection_set(&mut self, selection_set: &'a E::SelectionSet, r#type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>) {
                    $(self.[<$rule:snake>].visit_selection_set(selection_set, r#type);)*
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
);
