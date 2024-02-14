use bluejay_core::definition::{
    BaseInputTypeReference, InputType, InputTypeReference, SchemaDefinition,
    ShallowInputTypeReference,
};
use bluejay_core::executable::{VariableType, VariableTypeReference};

#[derive(Clone)]
pub enum VariableDefinitionInputType<'a, I: InputType> {
    Base(BaseInputTypeReference<'a, I>, bool),
    List(Box<Self>, bool),
}

impl<'a, I: InputType> InputType for VariableDefinitionInputType<'a, I> {
    type CustomScalarTypeDefinition = I::CustomScalarTypeDefinition;
    type EnumTypeDefinition = I::EnumTypeDefinition;
    type InputObjectTypeDefinition = I::InputObjectTypeDefinition;

    fn as_ref<
        S: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
    >(
        &self,
        _: &S,
    ) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required) => {
                InputTypeReference::Base(base.convert::<Self>(), *required)
            }
            Self::List(inner, required) => InputTypeReference::List(inner.as_ref(), *required),
        }
    }

    fn as_shallow_ref(&self) -> ShallowInputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required) => ShallowInputTypeReference::Base(base.name(), *required),
            Self::List(inner, required) => ShallowInputTypeReference::List(inner, *required),
        }
    }
}

impl<'a, S: SchemaDefinition, T: VariableType> TryFrom<(&'a S, &T)>
    for VariableDefinitionInputType<'a, S::InputType>
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

impl<'a, I: InputType, T: VariableType> TryFrom<(BaseInputTypeReference<'a, I>, &T)>
    for VariableDefinitionInputType<'a, I>
{
    type Error = ();

    fn try_from(
        (base, variable_type): (BaseInputTypeReference<'a, I>, &T),
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
