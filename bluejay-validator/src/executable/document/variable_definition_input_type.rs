use bluejay_core::definition::{BaseInputTypeReference, SchemaDefinition};
use bluejay_core::executable::{VariableType, VariableTypeReference};

#[derive(Clone)]
pub enum VariableDefinitionInputType<'a, S: SchemaDefinition> {
    Base(BaseInputTypeReference<'a, S>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinition, T: VariableType> TryFrom<(&'a S, &T)>
    for VariableDefinitionInputType<'a, S>
{
    type Error = ();

    fn try_from((schema_definition, variable_type): (&'a S, &T)) -> Result<Self, Self::Error> {
        let type_name = variable_type.as_ref().name();
        let type_definition_reference =
            schema_definition.get_type_definition(type_name).ok_or(())?;
        let base = BaseInputTypeReference::try_from(type_definition_reference)?;
        Self::try_from((base, variable_type))
    }
}

impl<'a, S: SchemaDefinition, T: VariableType> TryFrom<(BaseInputTypeReference<'a, S>, &T)>
    for VariableDefinitionInputType<'a, S>
{
    type Error = ();

    fn try_from(
        (base, variable_type): (BaseInputTypeReference<'a, S>, &T),
    ) -> Result<Self, Self::Error> {
        match variable_type.as_ref() {
            VariableTypeReference::Named(_, required) => {
                Ok(VariableDefinitionInputType::Base(base, required))
            }
            VariableTypeReference::List(inner, required) => Self::try_from((base, inner))
                .map(|inner| VariableDefinitionInputType::List(Box::new(inner), required)),
        }
    }
}
