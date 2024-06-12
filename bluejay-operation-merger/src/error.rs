use bluejay_core::executable::ExecutableDocument;

#[derive(Debug)]
pub enum Error<'a, E: ExecutableDocument> {
    OperationTypeMismatch {
        operation_name: Option<&'a str>,
    },
    FragmentDefinitionNotFound {
        fragment_name: &'a str,
    },
    DirectivesNotSupported, // TODO: Add more information
    ArgumentsNotCompatible {
        first: Option<&'a E::Arguments<false>>,
        second: Option<&'a E::Arguments<false>>,
    },
    DifferingFieldNamesForResponseName {
        response_name: &'a str,
    },
    VariableTypeMismatch {
        variable_name: &'a str,
    },
    VariableDefaultValueMismatch {
        variable_name: &'a str,
        first: Option<&'a E::Value<true>>,
        second: Option<&'a E::Value<true>>,
    },
}
