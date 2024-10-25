use crate::ast::executable::{
    ExecutableDefinition, ExplicitOperationDefinition, Field, FragmentDefinition, FragmentSpread,
    ImplicitOperationDefinition, InlineFragment, OperationDefinition, Selection, SelectionSet,
    VariableDefinition, VariableDefinitions, VariableType,
};
use crate::ast::{
    Argument, Arguments, DepthLimiter, Directive, Directives, Parse, ParseError, Tokens,
    TryFromTokens, Value,
};
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

    #[inline]
    fn is_empty(&self) -> bool {
        self.operation_definitions.is_empty() && self.fragment_definitions.is_empty()
    }
}

impl<'a> Parse<'a> for ExecutableDocument<'a> {
    #[inline]
    fn parse_from_tokens(
        mut tokens: impl Tokens<'a>,
        max_depth: usize,
    ) -> Result<Self, Vec<Error>> {
        let mut instance: Self = Self::new(Vec::new(), Vec::new());
        let mut errors = Vec::new();
        let mut last_pass_had_error = false;

        loop {
            last_pass_had_error = if let Some(res) =
                ExecutableDefinition::try_from_tokens(&mut tokens, DepthLimiter::new(max_depth))
            {
                match res {
                    Ok(ExecutableDefinition::Operation(operation_definition)) => {
                        instance.operation_definitions.push(operation_definition);
                        false
                    }
                    Ok(ExecutableDefinition::Fragment(fragment_definition)) => {
                        instance.fragment_definitions.push(fragment_definition);
                        false
                    }
                    Err(ParseError::MaxDepthExceeded) => {
                        errors.push(ParseError::MaxDepthExceeded);
                        // no sense in continuing to parse if we've hit the depth limit
                        break;
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

        let lex_errors = tokens.into_errors();

        let errors = if lex_errors.is_empty() {
            if errors.is_empty() && instance.is_empty() {
                vec![ParseError::EmptyDocument.into()]
            } else {
                errors.into_iter().map(Into::into).collect()
            }
        } else {
            lex_errors.into_iter().map(Into::into).collect()
        };

        if errors.is_empty() {
            Ok(instance)
        } else {
            Err(errors)
        }
    }
}

impl<'a> bluejay_core::executable::ExecutableDocument for ExecutableDocument<'a> {
    type Value<const CONST: bool> = Value<'a, CONST>;
    type VariableType = VariableType<'a>;
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
    type FragmentDefinitions<'b> = std::slice::Iter<'b, Self::FragmentDefinition> where Self: 'b;
    type OperationDefinitions<'b> = std::slice::Iter<'b, Self::OperationDefinition> where Self: 'b;

    fn operation_definitions(&self) -> Self::OperationDefinitions<'_> {
        self.operation_definitions.iter()
    }

    fn fragment_definitions(&self) -> Self::FragmentDefinitions<'_> {
        self.fragment_definitions.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutableDocument, Parse};
    use crate::ast::ParseOptions;

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

        let defs = ExecutableDocument::parse(document).unwrap();

        assert_eq!(2, defs.fragment_definitions().len());
        assert_eq!(1, defs.operation_definitions().len());
    }

    #[test]
    fn test_depth_limit() {
        // Depth is bumped to 1 entering the selection set (`{`)
        // Depth is bumped to 2 entering the field (`foo`)
        // Depth is bumped to 3 checking for args on the field (`foo`)
        let document = r#"query { foo }"#;

        let errors = ExecutableDocument::parse_with_options(
            document,
            ParseOptions {
                graphql_ruby_compatibility: false,
                max_depth: 2,
            },
        )
        .unwrap_err();

        assert_eq!(1, errors.len(), "{:?}", errors);

        assert_eq!("Max depth exceeded", errors[0].message());

        let executable_document = ExecutableDocument::parse_with_options(
            document,
            ParseOptions {
                graphql_ruby_compatibility: false,
                max_depth: 3,
            },
        )
        .unwrap();
        assert_eq!(1, executable_document.operation_definitions().len());
    }
}
