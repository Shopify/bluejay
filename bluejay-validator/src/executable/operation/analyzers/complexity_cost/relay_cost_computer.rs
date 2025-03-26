use crate::executable::operation::{
    analyzers::complexity_cost::{CostComputer, FieldMultipliers},
    OperationDefinitionValueEvaluationExt, VariableValues,
};
use bluejay_core::definition::{prelude::*, SchemaDefinition};
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::{Argument, AsIter, Directive, Value, ValueReference};
use std::marker::PhantomData;

const CONNECTION_COST_KIND: &str = "connection";
const CONNECTION_EDGES_FIELD: &str = "edges";
const CONNECTION_NODES_FIELD: &str = "nodes";
const CONNECTION_FIRST_ARGUMENT: &str = "first";
const CONNECTION_LAST_ARGUMENT: &str = "last";
const WEIGHT_ARGUMENT: &str = "weight";
const COST_ARGUMENT: &str = "cost";
const KIND_ARGUMENT: &str = "kind";

pub struct RelayCostComputer<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues> {
    operation_definition: &'a E::OperationDefinition,
    schema_definition: &'a S,
    variable_values: &'a V,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues> CostComputer<'a, E, S, V>
    for RelayCostComputer<'a, E, S, V>
{
    type FieldMultipliers = RelayFieldMultipliers<E>;

    fn new(
        operation_definition: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a V,
    ) -> Self {
        Self {
            operation_definition,
            schema_definition,
            variable_values,
        }
    }

    fn cost_for_field_definition(
        &self,
        field_definition: &<S as SchemaDefinition>::FieldDefinition,
    ) -> usize {
        let return_type = field_definition.r#type().base(self.schema_definition);

        field_definition
            .directives()
            .and_then(|directives| {
                directives
                    .iter()
                    .find(|directive| directive.name() == COST_ARGUMENT)
                    .and_then(|directive| directive.arguments())
                    .and_then(|arguments| {
                        arguments
                            .iter()
                            .find(|argument| argument.name() == WEIGHT_ARGUMENT)
                            .and_then(|argument| {
                                if let ValueReference::String(str) = argument.value().as_ref() {
                                    str.parse::<f32>()
                                        .ok()
                                        .map(|weight| weight.max(0f32) as usize)
                                } else {
                                    None
                                }
                            })
                    })
            })
            .unwrap_or_else(|| if return_type.is_composite() { 1 } else { 0 })
    }

    fn field_multipliers(
        &self,
        field_definition: &<S as SchemaDefinition>::FieldDefinition,
        field: &<E as ExecutableDocument>::Field,
    ) -> RelayFieldMultipliers<E> {
        let kind = field_definition.directives().and_then(|directives| {
            directives
                .iter()
                .find(|directive| directive.name() == COST_ARGUMENT)
                .and_then(|directive| directive.arguments())
                .and_then(|arguments| {
                    arguments
                        .iter()
                        .find(|argument| argument.name() == KIND_ARGUMENT)
                        .and_then(|argument| {
                            if let ValueReference::String(str) = argument.value().as_ref() {
                                Some(str)
                            } else {
                                None
                            }
                        })
                })
        });

        match kind {
            Some(CONNECTION_COST_KIND) => {
                let (first_size, last_size) = (
                    self.extract_field_sizing_argument(field, CONNECTION_FIRST_ARGUMENT),
                    self.extract_field_sizing_argument(field, CONNECTION_LAST_ARGUMENT),
                );

                let static_size = first_size.into_iter().chain(last_size).max().unwrap_or(0);

                let multiplier = Self::multiplier_for_static_size(static_size);

                RelayFieldMultipliers {
                    connection_multiplier: Some(multiplier),
                    executable_document: PhantomData,
                }
            }
            _ => RelayFieldMultipliers {
                connection_multiplier: None,
                executable_document: PhantomData,
            },
        }
    }
}

impl<E: ExecutableDocument, S: SchemaDefinition, V: VariableValues>
    RelayCostComputer<'_, E, S, V>
{
    fn extract_field_sizing_argument(
        &self,
        field: &<E as ExecutableDocument>::Field,
        argument_name: &str,
    ) -> Option<usize> {
        field
            .arguments()
            .and_then(|arguments| arguments.iter().find(|arg| arg.name() == argument_name))
            .and_then(|argument| match argument.value().as_ref() {
                ValueReference::Integer(int) => Some(int.max(0) as usize),
                ValueReference::Variable(var) => self
                    .operation_definition
                    .evaluate_int(var, self.variable_values)
                    .map(|i| i.max(0) as usize),
                _ => None,
            })
    }

    fn multiplier_for_static_size(static_size: usize) -> usize {
        if static_size > 0 {
            // floor(2 * ln(max(2, static_size)))
            (2f32 * (static_size.max(2) as f32).ln()).floor() as usize
        } else {
            0
        }
    }
}

pub struct RelayFieldMultipliers<E: ExecutableDocument> {
    connection_multiplier: Option<usize>,
    executable_document: PhantomData<E>,
}

impl<E: ExecutableDocument> Default for RelayFieldMultipliers<E> {
    fn default() -> Self {
        Self {
            connection_multiplier: None,
            executable_document: PhantomData,
        }
    }
}

impl<E: ExecutableDocument> FieldMultipliers<E> for RelayFieldMultipliers<E> {
    fn multiplier_for_field(&self, field: &E::Field) -> usize {
        let Some(multiplier) = self.connection_multiplier else {
            return 1;
        };
        match field.name() {
            CONNECTION_EDGES_FIELD | CONNECTION_NODES_FIELD => multiplier,
            _ => 1,
        }
    }
}
