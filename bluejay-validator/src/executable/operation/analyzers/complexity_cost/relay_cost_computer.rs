use crate::executable::operation::{
    analyzers::complexity_cost::{CostComputer, FieldMultipliers},
    OperationDefinitionValueEvaluationExt, VariableValues,
};
use bluejay_core::definition::{prelude::*, SchemaDefinition};
use bluejay_core::executable::{ExecutableDocument, Field, OperationDefinition};
use bluejay_core::{Argument, AsIter, Directive, IntegerValue, Value, ValueReference};
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
            Some(CONNECTION_COST_KIND) => RelayFieldMultipliers::for_connection(
                self.operation_definition,
                self.variable_values,
                field,
            ),
            _ => RelayFieldMultipliers::default(),
        }
    }
}

/// The multipliers a Relay connection field applies to its `edges` and `nodes`.
///
/// Cost computers that identify connections some other way than the `@cost`
/// directive can build these directly with [`Self::for_connection`].
pub struct RelayFieldMultipliers<E: ExecutableDocument> {
    connection_multiplier: Option<usize>,
    executable_document: PhantomData<E>,
}

impl<E: ExecutableDocument> RelayFieldMultipliers<E> {
    /// Multipliers for a connection field, sized by the larger of its `first`
    /// and `last` arguments. A variable argument resolves through
    /// `variable_values` and the operation's variable defaults.
    ///
    /// The multiplier is `floor(2 * ln(max(2, size)))`, `0` for a size of zero
    /// or below, and saturates for a size beyond `u64` so that an oversized
    /// page never costs less than an absent one.
    pub fn for_connection<V: VariableValues>(
        operation_definition: &E::OperationDefinition,
        variable_values: &V,
        field: &E::Field,
    ) -> Self {
        let connection_multiplier = [CONNECTION_FIRST_ARGUMENT, CONNECTION_LAST_ARGUMENT]
            .into_iter()
            .filter_map(|argument_name| {
                field_sizing_argument(operation_definition, variable_values, field, argument_name)
            })
            .map(multiplier_for_static_size)
            .max()
            .unwrap_or(0);

        Self {
            connection_multiplier: Some(connection_multiplier),
            executable_document: PhantomData,
        }
    }
}

fn field_sizing_argument<'a, F: Field, O: OperationDefinition, V: VariableValues>(
    operation_definition: &'a O,
    variable_values: &'a V,
    field: &'a F,
    argument_name: &str,
) -> Option<IntegerValue<'a>> {
    field
        .arguments()
        .and_then(|arguments| arguments.iter().find(|arg| arg.name() == argument_name))
        .and_then(|argument| match argument.value().as_ref() {
            ValueReference::Integer(int) => Some(int),
            ValueReference::Variable(var) => {
                operation_definition.evaluate_int(var, variable_values)
            }
            _ => None,
        })
}

fn multiplier_for_static_size(static_size: IntegerValue<'_>) -> usize {
    if static_size.is_negative() {
        return 0;
    }
    let Some(static_size) = static_size.as_u64() else {
        // An oversized positive integer must not look like an absent argument.
        // Conservatively saturate the cost rather than underestimate it.
        return usize::MAX;
    };
    if static_size > 0 {
        // floor(2 * ln(max(2, static_size)))
        (2f32 * (static_size.max(2) as f32).ln()).floor() as usize
    } else {
        0
    }
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
