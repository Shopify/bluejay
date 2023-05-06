use crate::executable::{SelectionSet, VariableDefinitions};
use crate::{OperationType, VariableDirectives};
use std::cmp::{Eq, Ord};
use std::hash::Hash;

#[derive(Debug)]
pub enum OperationDefinition<
    'a,
    E: ExplicitOperationDefinition,
    I: ImplicitOperationDefinition<SelectionSet = E::SelectionSet>,
> {
    Explicit(&'a E),
    Implicit(&'a I),
}

impl<
        'a,
        E: ExplicitOperationDefinition,
        I: ImplicitOperationDefinition<SelectionSet = E::SelectionSet>,
    > OperationDefinition<'a, E, I>
{
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

    pub fn variable_definitions(&self) -> Option<&'a E::VariableDefinitions> {
        match self {
            Self::Explicit(eod) => eod.variable_definitions(),
            Self::Implicit(_) => None,
        }
    }

    pub fn selection_set(&self) -> &'a E::SelectionSet {
        match self {
            Self::Explicit(eod) => eod.selection_set(),
            Self::Implicit(iod) => iod.selection_set(),
        }
    }

    pub fn directives(&self) -> Option<&'a E::Directives> {
        match self {
            Self::Explicit(eod) => Some(eod.directives()),
            Self::Implicit(_) => None,
        }
    }
}

pub trait AbstractOperationDefinition: Eq + Hash + Ord {
    type ExplicitOperationDefinition: ExplicitOperationDefinition;
    type ImplicitOperationDefinition: ImplicitOperationDefinition<SelectionSet=<Self::ExplicitOperationDefinition as ExplicitOperationDefinition>::SelectionSet>;

    fn as_ref(&self) -> OperationDefinitionFromAbstract<'_, Self>;
}

pub type OperationDefinitionFromAbstract<'a, O> = OperationDefinition<
    'a,
    <O as AbstractOperationDefinition>::ExplicitOperationDefinition,
    <O as AbstractOperationDefinition>::ImplicitOperationDefinition,
>;

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
