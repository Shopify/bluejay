use crate::definition::InputValueDefinition;
use crate::AsIter;

pub trait InputFieldsDefinition: AsIter<Item = Self::InputValueDefinition> {
    type InputValueDefinition: InputValueDefinition;

    fn get(&self, name: &str) -> Option<&Self::InputValueDefinition> {
        self.iter().find(|fd| fd.name() == name)
    }
}
