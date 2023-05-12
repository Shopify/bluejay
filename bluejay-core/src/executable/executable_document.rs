use crate::executable::{
    ExplicitOperationDefinition, Field, FragmentDefinition, FragmentSpread,
    ImplicitOperationDefinition, InlineFragment, OperationDefinition, Selection, SelectionSet,
    VariableDefinition, VariableDefinitions, VariableType,
};
use crate::{Argument, Arguments, Directive, Directives, Value};

pub trait ExecutableDocument {
    type Value<const CONST: bool>: Value<CONST>;
    type VariableType: VariableType;
    type Argument<const CONST: bool>: Argument<CONST, Value = Self::Value<CONST>>;
    type Arguments<const CONST: bool>: Arguments<CONST, Argument = Self::Argument<CONST>>;
    type Directive<const CONST: bool>: Directive<CONST, Arguments = Self::Arguments<CONST>>;
    type Directives<const CONST: bool>: Directives<CONST, Directive = Self::Directive<CONST>>;
    type FragmentSpread: FragmentSpread<Directives = Self::Directives<false>>;
    type Field: Field<
        Arguments = Self::Arguments<false>,
        Directives = Self::Directives<false>,
        SelectionSet = Self::SelectionSet,
    >;
    type Selection: Selection<
        Field = Self::Field,
        FragmentSpread = Self::FragmentSpread,
        InlineFragment = Self::InlineFragment,
    >;
    type SelectionSet: SelectionSet<Selection = Self::Selection>;
    type InlineFragment: InlineFragment<
        Directives = Self::Directives<false>,
        SelectionSet = Self::SelectionSet,
    >;
    type VariableDefinition: VariableDefinition<
        VariableType = Self::VariableType,
        Directives = Self::Directives<true>,
        Value = Self::Value<true>,
    >;
    type VariableDefinitions: VariableDefinitions<VariableDefinition = Self::VariableDefinition>;
    type ExplicitOperationDefinition: ExplicitOperationDefinition<
        VariableDefinitions = Self::VariableDefinitions,
        Directives = Self::Directives<false>,
        SelectionSet = Self::SelectionSet,
    >;
    type ImplicitOperationDefinition: ImplicitOperationDefinition<SelectionSet = Self::SelectionSet>;
    type OperationDefinition: OperationDefinition<
        ExplicitOperationDefinition = Self::ExplicitOperationDefinition,
        ImplicitOperationDefinition = Self::ImplicitOperationDefinition,
    >;
    type FragmentDefinition: FragmentDefinition<
        Directives = Self::Directives<false>,
        SelectionSet = Self::SelectionSet,
    >;

    fn operation_definitions(&self) -> &[Self::OperationDefinition];
    fn fragment_definitions(&self) -> &[Self::FragmentDefinition];
}
