use crate::executable::SelectionSet;
use crate::VariableDirectives;

pub trait InlineFragment {
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn type_condition(&self) -> Option<&str>;
    fn directives(&self) -> &Self::Directives;
    fn selection_set(&self) -> &Self::SelectionSet;
}
