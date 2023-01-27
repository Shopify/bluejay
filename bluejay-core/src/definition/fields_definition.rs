use crate::definition::FieldDefinition;
use crate::AsIter;

pub trait FieldsDefinition: AsIter<Item=Self::FieldDefinition> {
    type FieldDefinition: FieldDefinition;

    fn contains_field(&self, name: &str) -> bool {
        self.iter().any(|fd| fd.name() == name)
    }

    fn get_field(&self, name: &str) -> Option<&Self::FieldDefinition> {
        self.iter().find(|fd| fd.name() == name)
    }
}
