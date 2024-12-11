mod error;
mod fragment_and_operation_names_do_not_clash;
mod selections_are_valid;

use error::Error;
use fragment_and_operation_names_do_not_clash::FragmentAndOperationNamesDoNotClash;
use selections_are_valid::SelectionsAreValid;

pub(crate) type Rule<'a, E, S> = (
    SelectionsAreValid<'a, E, S>,
    FragmentAndOperationNamesDoNotClash<'a, E, S>,
);
