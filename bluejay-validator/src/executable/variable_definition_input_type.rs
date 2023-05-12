use bluejay_core::definition::{
    BaseInputType, BaseInputTypeReference, InputType, InputTypeReference, SchemaDefinition,
};
use bluejay_core::executable::{VariableType, VariableTypeReference};

#[derive(Clone)]
pub enum VariableDefinitionInputType<'a, B: BaseInputType> {
    Base(BaseInputTypeReference<'a, B>, bool),
    List(Box<Self>, bool),
}

impl<'a, B: BaseInputType> InputType for VariableDefinitionInputType<'a, B> {
    type BaseInputType = BaseInputTypeReference<'a, B>;

    fn as_ref(&self) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required) => InputTypeReference::Base(base, *required),
            Self::List(inner, required) => InputTypeReference::List(inner.as_ref(), *required),
        }
    }
}

impl<'a, S: SchemaDefinition, T: VariableType> TryFrom<(&'a S, &T)>
    for VariableDefinitionInputType<'a, S::BaseInputType>
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
    for VariableDefinitionInputType<'a, B>
{
    type Error = ();

    fn try_from(
        (base, type_reference): (BaseInputTypeReference<'a, B>, &T),
    ) -> Result<Self, Self::Error> {
        match type_reference.as_ref() {
            VariableTypeReference::Named(_, required) => {
                Ok(VariableDefinitionInputType::Base(base, required))
            }
            VariableTypeReference::List(inner, required) => Self::try_from((base, inner))
                .map(|inner| VariableDefinitionInputType::List(Box::new(inner), required)),
        }
    }
}
