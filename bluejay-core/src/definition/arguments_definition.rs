use crate::definition::InputValueDefinition;
use crate::AsIter;

pub trait ArgumentsDefinition: AsIter<Item = Self::ArgumentDefinition> {
    type ArgumentDefinition: InputValueDefinition;

    fn get(&self, name: &str) -> Option<&Self::ArgumentDefinition> {
        self.iter().find(|fd| fd.name() == name)
    }
}
