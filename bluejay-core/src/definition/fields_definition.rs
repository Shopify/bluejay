use crate::definition::FieldDefinition;
use crate::AsIter;

pub trait FieldsDefinition: AsIter<Item = Self::FieldDefinition> {
    type FieldDefinition: FieldDefinition;

    #[inline]
    fn contains_field(&self, name: &str) -> bool {
        self.iter().any(|fd| fd.name() == name)
    }

    #[inline]
    fn get(&self, name: &str) -> Option<&Self::FieldDefinition> {
        self.iter().find(|fd| fd.name() == name)
    }
}
