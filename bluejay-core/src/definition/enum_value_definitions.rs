use crate::definition::EnumValueDefinition;
use crate::AsIter;

pub trait EnumValueDefinitions: AsIter<Item=Self::EnumValueDefinition> {
    type EnumValueDefinition: EnumValueDefinition;
}
