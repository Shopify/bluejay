mod argument;
mod arguments;
mod directive;
mod directives;
pub mod executable;
mod from_tokens;
mod is_match;
mod operation_type;
mod parse_error;
mod tokens;
mod try_from_tokens;
mod type_reference;
mod value;
mod variable;

pub use argument::{Argument, ConstArgument, VariableArgument};
use arguments::{Arguments, VariableArguments};
use directive::Directive;
use directives::{ConstDirectives, Directives, VariableDirectives};
use from_tokens::FromTokens;
use is_match::IsMatch;
use parse_error::ParseError;
use tokens::{ScannerTokens, Tokens};
use try_from_tokens::TryFromTokens;
pub use type_reference::TypeReference;
pub use value::{ConstValue, Value, VariableValue};
use variable::Variable;

pub fn parse<'a>(s: &'a str) -> (executable::ExecutableDocument<'a>, Vec<crate::error::Error>) {
    let scanner = crate::LogosScanner::new(s);
    let mut tokens = ScannerTokens::new(scanner);
    let mut operation_definitions: Vec<executable::OperationDefinition<'a>> = Vec::new();
    let mut fragment_definitions: Vec<executable::FragmentDefinition<'a>> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    loop {
        if let Some(res) = executable::ExecutableDefinition::try_from_tokens(&mut tokens) {
            match res {
                Ok(executable::ExecutableDefinition::Operation(operation_definition)) => {
                    operation_definitions.push(operation_definition)
                }
                Ok(executable::ExecutableDefinition::Fragment(fragment_definition)) => {
                    fragment_definitions.push(fragment_definition)
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        } else if let Some(token) = tokens.next() {
            errors.push(ParseError::UnexpectedToken { span: token.into() })
        } else {
            break;
        }
    }

    let errors = errors
        .into_iter()
        .map(|e| e.into())
        .chain(tokens.errors.into_iter().map(|e| e.into()))
        .collect();

    let executable_document =
        executable::ExecutableDocument::new(operation_definitions, fragment_definitions);

    (executable_document, errors)
}

#[cfg(test)]
mod tests {
    use super::parse;

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

        let (defs, errs) = parse(document);

        dbg!(&defs);

        assert_eq!(Vec::<crate::error::Error>::new(), errs);
        assert_eq!(2, defs.fragment_definitions().len());
        assert_eq!(1, defs.operation_definitions().len());
    }
}
