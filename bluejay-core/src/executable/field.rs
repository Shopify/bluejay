use crate::executable::SelectionSet;
use crate::{VariableArguments, VariableDirectives};

pub trait Field {
    type Arguments: VariableArguments;
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn alias(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments(&self) -> Option<&Self::Arguments>;
    fn directives(&self) -> Option<&Self::Directives>;
    fn selection_set(&self) -> Option<&Self::SelectionSet>;

    fn response_name(&self) -> &str {
        self.alias().unwrap_or(self.name())
    }
}
