use crate::VariableDirectives;
use crate::executable::SelectionSet;

pub trait FragmentDefinition {
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn name(&self) -> &str;
    fn type_condition(&self) -> &str;
    fn directives(&self) -> &Self::Directives;
    fn selection_set(&self) -> &Self::SelectionSet;
}
