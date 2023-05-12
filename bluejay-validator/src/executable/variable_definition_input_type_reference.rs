use bluejay_core::definition::{
    AbstractInputTypeReference, BaseInputType, BaseInputTypeReference,
    InputTypeReference as CoreInputTypeReference, InputTypeReferenceFromAbstract, SchemaDefinition,
};
use bluejay_core::executable::{VariableType, VariableTypeReference as CoreTypeReference};

#[derive(Clone)]
pub enum VariableDefinitionInputTypeReference<'a, B: BaseInputType> {
    Base(BaseInputTypeReference<'a, B>, bool),
    List(Box<Self>, bool),
}

impl<'a, B: BaseInputType> AbstractInputTypeReference
    for VariableDefinitionInputTypeReference<'a, B>
{
    type BaseInputType = BaseInputTypeReference<'a, B>;

    fn as_ref(&self) -> InputTypeReferenceFromAbstract<'_, Self> {
        match self {
            Self::Base(base, required) => CoreInputTypeReference::Base(base, *required),
            Self::List(inner, required) => CoreInputTypeReference::List(inner.as_ref(), *required),
        }
    }
}

impl<'a, S: SchemaDefinition, T: VariableType> TryFrom<(&'a S, &T)>
    for VariableDefinitionInputTypeReference<'a, S::BaseInputType>
{
    type Error = ();

    fn try_from((schema_definition, type_reference): (&'a S, &T)) -> Result<Self, Self::Error> {
        let type_name = type_reference.as_ref().name();
        let type_definition_reference =
            schema_definition.get_type_definition(type_name).ok_or(())?;
        let base = BaseInputTypeReference::try_from(type_definition_reference)?;
        Self::try_from((base, type_reference))
    }
}

impl<'a, B: BaseInputType, T: VariableType> TryFrom<(BaseInputTypeReference<'a, B>, &T)>
    for VariableDefinitionInputTypeReference<'a, B>
{
    type Error = ();

    fn try_from(
        (base, type_reference): (BaseInputTypeReference<'a, B>, &T),
    ) -> Result<Self, Self::Error> {
        match type_reference.as_ref() {
            CoreTypeReference::Named(_, required) => {
                Ok(VariableDefinitionInputTypeReference::Base(base, required))
            }
            CoreTypeReference::List(inner, required) => Self::try_from((base, inner))
                .map(|inner| VariableDefinitionInputTypeReference::List(Box::new(inner), required)),
        }
    }
}
