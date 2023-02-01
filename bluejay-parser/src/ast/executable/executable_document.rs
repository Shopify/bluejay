use crate::ast::executable::{
    ExecutableDefinition, ExplicitOperationDefinition, Field, FragmentDefinition, FragmentSpread,
    ImplicitOperationDefinition, InlineFragment, OperationDefinition, Selection, SelectionSet,
    VariableDefinition, VariableDefinitions,
};
use crate::ast::{Argument, Arguments, Directive, Directives, Value, Variable};

#[derive(Debug)]
pub struct ExecutableDocument<'a> {
    operation_definitions: Vec<OperationDefinition<'a>>,
    fragment_definitions: Vec<FragmentDefinition<'a>>,
}

impl<'a> ExecutableDocument<'a> {
    pub(crate) fn new(
        operation_definitions: Vec<OperationDefinition<'a>>,
        fragment_definitions: Vec<FragmentDefinition<'a>>,
    ) -> Self {
        Self {
            operation_definitions,
            fragment_definitions,
        }
    }

    pub fn operation_definitions(&self) -> &[OperationDefinition<'a>] {
        &self.operation_definitions
    }

    pub fn fragment_definitions(&self) -> &[FragmentDefinition<'a>] {
        &self.fragment_definitions
    }
}

impl<'a> bluejay_core::executable::ExecutableDocument<'a> for ExecutableDocument<'a> {
    type Variable = Variable<'a>;
    type Value<const CONST: bool> = Value<'a, CONST>;
    type TypeReference = crate::ast::TypeReference<'a>;
    type Argument<const CONST: bool> = Argument<'a, CONST>;
    type Arguments<const CONST: bool> = Arguments<'a, CONST>;
    type Directive<const CONST: bool> = Directive<'a, CONST>;
    type Directives<const CONST: bool> = Directives<'a, CONST>;
    type FragmentSpread = FragmentSpread<'a>;
    type Field = Field<'a>;
    type Selection = Selection<'a>;
    type SelectionSet = SelectionSet<'a>;
    type InlineFragment = InlineFragment<'a>;
    type VariableDefinition = VariableDefinition<'a>;
    type VariableDefinitions = VariableDefinitions<'a>;
    type ExplicitOperationDefinition = ExplicitOperationDefinition<'a>;
    type ImplicitOperationDefinition = ImplicitOperationDefinition<'a>;
    type OperationDefinition = OperationDefinition<'a>;
    type FragmentDefinition = FragmentDefinition<'a>;
    type ExecutableDefinition = ExecutableDefinition<'a>;

    fn operation_definitions(&self) -> &[Self::OperationDefinition] {
        &self.operation_definitions
    }

    fn fragment_definitions(&self) -> &[Self::FragmentDefinition] {
        &self.fragment_definitions
    }
}
