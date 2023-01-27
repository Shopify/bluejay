use crate::AsIter;
use crate::definition::InputValueDefinition;

pub trait InputFieldsDefinition: AsIter<Item=Self::InputValueDefinition> {
    type InputValueDefinition: InputValueDefinition;
}
