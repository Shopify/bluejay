use crate::executable::{AbstractOperationDefinition, FragmentDefinition};

#[derive(Debug)]
pub enum ExecutableDefinition<O: AbstractOperationDefinition, F: FragmentDefinition> {
    Operation(O),
    Fragment(F),
}

pub trait AbstractExecutableDefinition: Into<ExecutableDefinition<Self::OperationDefinition, Self::FragmentDefinition>> {
    type OperationDefinition: AbstractOperationDefinition;
    type FragmentDefinition: FragmentDefinition;
}

impl<O: AbstractOperationDefinition, F: FragmentDefinition> AbstractExecutableDefinition for ExecutableDefinition<O, F> {
    type OperationDefinition = O;
    type FragmentDefinition = F;
}
