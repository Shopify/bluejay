use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait ScaleFactorComputer<'a, E: ExecutableDocument, S: SchemaDefinition> {
    type FieldMultipliers: FieldMultipliers<E>;

    fn new(operation_definition: &'a E::OperationDefinition, schema_definition: &'a S) -> Self;

    fn cost_for_field_definition(&self, field_definition: &S::FieldDefinition) -> usize;

    fn field_multipliers(
        &self,
        field_definition: &S::FieldDefinition,
        field: &E::Field,
    ) -> Self::FieldMultipliers;
}

pub trait FieldMultipliers<E: ExecutableDocument>: Default {
    fn multiplier_for_field(&self, field: &E::Field) -> usize;
}

pub struct DefaultScaleFactorComputer;

impl<'a, E: ExecutableDocument, S: SchemaDefinition> ScaleFactorComputer<'a, E, S>
    for DefaultScaleFactorComputer
{
    type FieldMultipliers = UnitFieldMultipliers;

    fn new(_: &'a E::OperationDefinition, _: &'a S) -> Self {
        Self
    }

    fn cost_for_field_definition(&self, _: &<S as SchemaDefinition>::FieldDefinition) -> usize {
        1
    }

    fn field_multipliers(
        &self,
        _: &<S as SchemaDefinition>::FieldDefinition,
        _: &<E as ExecutableDocument>::Field,
    ) -> UnitFieldMultipliers {
        UnitFieldMultipliers
    }
}

#[derive(Default)]
pub struct UnitFieldMultipliers;

impl<E: ExecutableDocument> FieldMultipliers<E> for UnitFieldMultipliers {
    fn multiplier_for_field(&self, _: &E::Field) -> usize {
        1
    }
}
