mod error;
mod fragment_and_operation_names_do_not_clash;
mod paths_with_custom_scalar_type;
mod selections_are_valid;
mod variables_struct_names_do_not_clash;

use error::Error;
use fragment_and_operation_names_do_not_clash::FragmentAndOperationNamesDoNotClash;
use selections_are_valid::SelectionsAreValid;
use variables_struct_names_do_not_clash::VariablesStructNamesDoNotClash;

pub(crate) type Rule<'a, E, S> = (
    SelectionsAreValid<'a, E, S>,
    FragmentAndOperationNamesDoNotClash<'a, E, S>,
    VariablesStructNamesDoNotClash<'a, E, S>,
);

pub(crate) use paths_with_custom_scalar_type::PathsWithCustomScalarType;
