use crate::executable::{SelectionSet, VariableDefinitions};
use crate::{Indexable, OperationType, VariableDirectives};

#[derive(Debug)]
pub enum OperationDefinitionReference<'a, O: OperationDefinition> {
    Explicit(&'a O::ExplicitOperationDefinition),
    Implicit(&'a O::ImplicitOperationDefinition),
}

impl<'a, O: OperationDefinition> OperationDefinitionReference<'a, O> {
    pub fn operation_type(&self) -> OperationType {
        match self {
            Self::Explicit(eod) => eod.operation_type(),
            Self::Implicit(_) => OperationType::Query,
        }
    }

    pub fn name(&self) -> Option<&'a str> {
        match self {
            Self::Explicit(eod) => eod.name(),
            Self::Implicit(_) => None,
        }
    }

    pub fn variable_definitions(&self) -> Option<&'a <<O as OperationDefinition>::ExplicitOperationDefinition as ExplicitOperationDefinition>::VariableDefinitions>{
        match self {
            Self::Explicit(eod) => eod.variable_definitions(),
            Self::Implicit(_) => None,
        }
    }

    pub fn selection_set(&self) -> &'a <<O as OperationDefinition>::ExplicitOperationDefinition as ExplicitOperationDefinition>::SelectionSet{
        match self {
            Self::Explicit(eod) => eod.selection_set(),
            Self::Implicit(iod) => iod.selection_set(),
        }
    }

    pub fn directives(&self) -> Option<&'a <<O as OperationDefinition>::ExplicitOperationDefinition as ExplicitOperationDefinition>::Directives>{
        match self {
            Self::Explicit(eod) => Some(eod.directives()),
            Self::Implicit(_) => None,
        }
    }
}

pub trait OperationDefinition: Sized + Indexable {
    type ExplicitOperationDefinition: ExplicitOperationDefinition;
    type ImplicitOperationDefinition: ImplicitOperationDefinition<SelectionSet=<Self::ExplicitOperationDefinition as ExplicitOperationDefinition>::SelectionSet>;

    fn as_ref(&self) -> OperationDefinitionReference<'_, Self>;
}

pub trait ExplicitOperationDefinition {
    type VariableDefinitions: VariableDefinitions;
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn operation_type(&self) -> OperationType;
    fn name(&self) -> Option<&str>;
    fn variable_definitions(&self) -> Option<&Self::VariableDefinitions>;
    fn directives(&self) -> &Self::Directives;
    fn selection_set(&self) -> &Self::SelectionSet;
}

pub trait ImplicitOperationDefinition {
    type SelectionSet: SelectionSet;

    fn selection_set(&self) -> &Self::SelectionSet;
}
