use std::borrow::Cow;

#[derive(Debug)]
pub enum Error<'a> {
    OperationTypeMismatch { operation_name: Option<&'a str> },
    FragmentDefinitionNotFound { fragment_name: &'a str },
    DirectivesNotSupported, // TODO: Add more information
    ArgumentsNotCompatible, // TODO: Add more information
    DifferingFieldNamesForResponseName { response_name: &'a str },
    VariableTypeMismatch { variable_name: Cow<'a, str> },
    VariableDefaultValueMismatch { variable_name: Cow<'a, str> },
}
