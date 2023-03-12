use crate::executable::{ExecutableDocument, SelectionSet, VariableDefinitions};
use crate::{OperationType, VariableDirectives};

#[derive(Debug)]
pub enum OperationDefinition<
    E: ExplicitOperationDefinition,
    I: ImplicitOperationDefinition<SelectionSet = E::SelectionSet>,
> {
    Explicit(E),
    Implicit(I),
}

impl<
        E: ExplicitOperationDefinition,
        I: ImplicitOperationDefinition<SelectionSet = E::SelectionSet>,
    > OperationDefinition<E, I>
{
    pub fn operation_type(&self) -> OperationType {
        match self {
            Self::Explicit(eod) => eod.operation_type(),
            Self::Implicit(_) => OperationType::Query,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Explicit(eod) => eod.name(),
            Self::Implicit(_) => None,
        }
    }

    pub fn variable_definitions(&self) -> Option<&E::VariableDefinitions> {
        match self {
            Self::Explicit(eod) => eod.variable_definitions(),
            Self::Implicit(_) => None,
        }
    }

    pub fn selection_set(&self) -> &E::SelectionSet {
        match self {
            Self::Explicit(eod) => eod.selection_set(),
            Self::Implicit(iod) => iod.selection_set(),
        }
    }

    pub fn directives(&self) -> Option<&E::Directives> {
        match self {
            Self::Explicit(eod) => Some(eod.directives()),
            Self::Implicit(_) => None,
        }
    }
}

pub trait AbstractOperationDefinition: Into<OperationDefinition<Self::ExplicitOperationDefinition, Self::ImplicitOperationDefinition>>
    + AsRef<OperationDefinition<Self::ExplicitOperationDefinition, Self::ImplicitOperationDefinition>>
{
    type ExplicitOperationDefinition: ExplicitOperationDefinition;
    type ImplicitOperationDefinition: ImplicitOperationDefinition<SelectionSet=<Self::ExplicitOperationDefinition as ExplicitOperationDefinition>::SelectionSet>;
}

impl<
        E: ExplicitOperationDefinition,
        I: ImplicitOperationDefinition<SelectionSet = E::SelectionSet>,
    > AsRef<OperationDefinition<E, I>> for OperationDefinition<E, I>
{
    fn as_ref(&self) -> &OperationDefinition<E, I> {
        self
    }
}

impl<
        E: ExplicitOperationDefinition,
        I: ImplicitOperationDefinition<SelectionSet = E::SelectionSet>,
    > AbstractOperationDefinition for OperationDefinition<E, I>
{
    type ExplicitOperationDefinition = E;
    type ImplicitOperationDefinition = I;
}

pub type OperationDefinitionFromExecutableDocument<E> = OperationDefinition<
    <E as ExecutableDocument>::ExplicitOperationDefinition,
    <E as ExecutableDocument>::ImplicitOperationDefinition,
>;

pub trait ExplicitOperationDefinition: Sized {
    type VariableDefinitions: VariableDefinitions;
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn operation_type(&self) -> OperationType;
    fn name(&self) -> Option<&str>;
    fn variable_definitions(&self) -> Option<&Self::VariableDefinitions>;
    fn directives(&self) -> &Self::Directives;
    fn selection_set(&self) -> &Self::SelectionSet;
}

pub trait ImplicitOperationDefinition: Sized {
    type SelectionSet: SelectionSet;

    fn selection_set(&self) -> &Self::SelectionSet;
}
