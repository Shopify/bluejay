use crate::definition::InputValueDefinition;
use crate::AsIter;

pub trait InputFieldsDefinition: AsIter<Item = Self::InputValueDefinition> {
    type InputValueDefinition: InputValueDefinition;
}
