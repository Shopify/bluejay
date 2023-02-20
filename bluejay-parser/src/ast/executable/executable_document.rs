use crate::ast::executable::{
    ExecutableDefinition, ExplicitOperationDefinition, Field, FragmentDefinition, FragmentSpread,
    ImplicitOperationDefinition, InlineFragment, OperationDefinition, Selection, SelectionSet,
    VariableDefinition, VariableDefinitions,
};
use crate::ast::{
    Argument, Arguments, Directive, Directives, ParseError, ScannerTokens, TryFromTokens, Value,
    Variable,
};
use crate::scanner::LogosScanner;
use crate::Error;

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

    pub fn parse(s: &'a str) -> (Self, Vec<Error>) {
        let scanner = LogosScanner::new(s);
        let mut tokens = ScannerTokens::new(scanner);

        let mut instance: Self = Self::new(Vec::new(), Vec::new());
        let mut errors = Vec::new();
        let mut last_pass_had_error = false;

        loop {
            last_pass_had_error =
                if let Some(res) = ExecutableDefinition::try_from_tokens(&mut tokens) {
                    match res {
                        Ok(ExecutableDefinition::Operation(operation_definition)) => {
                            instance.operation_definitions.push(operation_definition);
                            false
                        }
                        Ok(ExecutableDefinition::Fragment(fragment_definition)) => {
                            instance.fragment_definitions.push(fragment_definition);
                            false
                        }
                        Err(err) => {
                            if !last_pass_had_error {
                                errors.push(err);
                            }
                            true
                        }
                    }
                } else if let Some(token) = tokens.next() {
                    if !last_pass_had_error {
                        errors.push(ParseError::UnexpectedToken { span: token.into() });
                    }
                    true
                } else {
                    break;
                }
        }

        let errors = if tokens.errors.is_empty() {
            if errors.is_empty() && instance.is_empty() {
                vec![ParseError::EmptyDocument.into()]
            } else {
                errors.into_iter().map(Into::into).collect()
            }
        } else {
            tokens.errors.into_iter().map(Into::into).collect()
        };

        (instance, errors)
    }

    fn is_empty(&self) -> bool {
        self.operation_definitions.is_empty() && self.fragment_definitions.is_empty()
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

#[cfg(test)]
mod tests {
    use super::ExecutableDocument;

    #[test]
    fn test_success() {
        let document = r#"
            {
                dog {
                    ...fragmentOne
                    ...fragmentTwo
                }
            }

            fragment fragmentOne on Dog {
                name
            }

            fragment fragmentTwo on Dog {
                owner {
                    name
                }
            }
        "#;

        let (defs, errs) = ExecutableDocument::parse(document);

        assert_eq!(0, errs.len());
        assert_eq!(2, defs.fragment_definitions().len());
        assert_eq!(1, defs.operation_definitions().len());
    }
}
