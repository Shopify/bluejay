use crate::definition::InputValueDefinition;
use crate::AsIter;

pub trait ArgumentsDefinition: AsIter<Item=Self::ArgumentDefinition> {
    type ArgumentDefinition: InputValueDefinition;
}
