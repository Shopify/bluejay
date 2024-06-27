use std::collections::HashMap;

use crate::executable::{
    operation::{Analyzer, VariableValues, Visitor},
    Cache,
};
use bluejay_core::{
    definition::SchemaDefinition, executable::ExecutableDocument, Argument, AsIter, ObjectValue,
    Value, ValueReference, Variable,
};

#[derive(Clone)]
pub struct Offender {
    pub size: usize,
    pub name: String,
}

#[derive(Clone)]
pub struct InputSize<'a, VV: VariableValues> {
    offenders: Vec<Offender>,
    max_length: usize,
    variable_values: HashMap<&'a str, (&'a VV::Key, &'a VV::Value)>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues>
    Visitor<'a, E, S, VV, usize> for InputSize<'a, VV>
{
    fn new(
        _: &'a E::OperationDefinition,
        _s: &'a S,
        variables: &'a VV,
        _: &'a Cache<'a, E, S>,
        max_length: usize,
    ) -> Self {
        Self {
            max_length,
            offenders: vec![],
            variable_values: variables
                .iter()
                .map(|(key, value)| (key.as_ref(), (key, value)))
                .collect(),
        }
    }

    fn visit_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<false>,
        _input_value_definition: &'a S::InputValueDefinition,
    ) {
        find_input_size_offenders_arguments::<E, VV>(
            self.max_length,
            &mut self.offenders,
            &self.variable_values,
            argument.name().to_string(),
            argument.value(),
        );
    }
}

fn find_input_size_offenders_arguments<'a, E: ExecutableDocument, VV: VariableValues>(
    max_length: usize,
    offenders: &mut Vec<Offender>,
    variable_values: &HashMap<&'a str, (&'a VV::Key, &'a VV::Value)>,
    argument_name: String,
    argument_value: &<E as bluejay_core::executable::ExecutableDocument>::Value<false>,
) {
    match argument_value.as_ref() {
        ValueReference::List(list) => {
            let list_length = list.len();
            if list_length > max_length {
                offenders.push(Offender {
                    size: list_length,
                    name: argument_name.to_string(),
                })
            } else {
                list.iter().enumerate().for_each(|(index, item)| {
                    find_input_size_offenders_arguments::<E, VV>(
                        max_length,
                        offenders,
                        variable_values,
                        format!("{}.{}", argument_name, index),
                        item,
                    );
                })
            }
        }
        ValueReference::Object(obj) => {
            // Traverse into children
            obj.iter().for_each(|(key, value)| {
                find_input_size_offenders_arguments::<E, VV>(
                    max_length,
                    offenders,
                    variable_values,
                    format!("{}.{}", argument_name, key.as_ref()),
                    value,
                );
            });
        }
        ValueReference::Variable(var) => {
            let name = var.name();
            // Get the variable and do this again. also does not work...
            let variable = variable_values.get(name);
            if let Some((_, value)) = variable {
                find_input_size_offenders_variables::<E, VV>(
                    max_length,
                    offenders,
                    variable_values,
                    argument_name,
                    *value,
                );
            }
        }
        _ => {}
    };
}

fn find_input_size_offenders_variables<'a, E: ExecutableDocument, VV: VariableValues>(
    max_length: usize,
    offenders: &mut Vec<Offender>,
    variable_values: &HashMap<&'a str, (&'a VV::Key, &'a VV::Value)>,
    argument_name: String,
    argument_value: &VV::Value,
) {
    match argument_value.as_ref() {
        ValueReference::List(list) => {
            let list_length = list.len();
            if list_length > max_length {
                offenders.push(Offender {
                    size: list_length,
                    name: argument_name.to_string(),
                })
            } else {
                list.iter().enumerate().for_each(|(index, item)| {
                    find_input_size_offenders_variables::<E, VV>(
                        max_length,
                        offenders,
                        variable_values,
                        format!("{}.{}", argument_name, index),
                        item,
                    );
                })
            }
        }
        ValueReference::Object(obj) => {
            // Traverse into children
            obj.iter().for_each(|(key, value)| {
                find_input_size_offenders_variables::<E, VV>(
                    max_length,
                    offenders,
                    variable_values,
                    format!("{}.{}", argument_name, key.as_ref()),
                    value,
                );
            });
        }
        ValueReference::Variable(var) => {
            let name = var.name();
            // Get the variable and do this again. also does not work...
            let variable = variable_values.get(name);
            if let Some((_, value)) = variable {
                find_input_size_offenders_variables::<E, VV>(
                    max_length,
                    offenders,
                    variable_values,
                    argument_name,
                    *value,
                );
            }
        }
        _ => {}
    };
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues>
    Analyzer<'a, E, S, VV, usize> for InputSize<'a, VV>
{
    type Output = Vec<Offender>;

    fn into_output(self) -> Self::Output {
        self.offenders
    }
}

#[cfg(test)]
mod tests {
    use super::{InputSize, Offender};
    use crate::executable::{operation::Orchestrator, Cache};
    use bluejay_parser::ast::{
        definition::{
            DefaultContext, DefinitionDocument, SchemaDefinition as ParserSchemaDefinition,
        },
        executable::ExecutableDocument as ParserExecutableDocument,
        Parse,
    };
    use serde_json::{Map as JsonMap, Value as JsonValue};

    const TEST_SCHEMA: &str = r#"
        input ObjectList {
            property: [String]
        }
        type Query {
          simple(x: [String]): String!
          object(x: ObjectList): String!
          list_object(x: [ObjectList]): String!
        }
        schema {
          query: Query
        }
    "#;

    fn get_size(query: String, variables: serde_json::Value) -> Vec<Offender> {
        let definition_document: DefinitionDocument<'_, DefaultContext> =
            DefinitionDocument::parse(TEST_SCHEMA).expect("Schema had parse errors");
        let schema_definition =
            ParserSchemaDefinition::try_from(&definition_document).expect("Schema had errors");
        let executable_document = ParserExecutableDocument::parse(&query)
            .unwrap_or_else(|_| panic!("Document had parse errors"));
        let cache = Cache::new(&executable_document, &schema_definition);
        let variables = variables.as_object().expect("Variables must be an object");
        Orchestrator::<_, _, JsonMap<String, JsonValue>, usize, InputSize<_>>::analyze(
            &executable_document,
            &schema_definition,
            None,
            variables,
            &cache,
            1,
        )
        .unwrap()
    }

    #[test]
    fn simple_size() {
        let result = get_size(
            r#"query { simple(x: ["x", "y"])} "#.to_string(),
            serde_json::json!({}),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x");
    }

    #[test]
    fn simple_size_variable() {
        let result = get_size(
            r#"query ($x: [String]) { simple(x: $x)} "#.to_string(),
            serde_json::json!({ "x": ["x", "y"] }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x");
    }

    #[test]
    fn object_size() {
        let result = get_size(
            r#"query { object(x: { property: ["x", "y"] })} "#.to_string(),
            serde_json::json!({}),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x.property");
    }

    #[test]
    fn object_size_variable() {
        let result = get_size(
            r#"query($x: ObjectList) { object(x: $x)} "#.to_string(),
            serde_json::json!({ "x": { "property": ["x", "y"] } }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x.property");
    }

    #[test]
    fn list_object_size() {
        let result = get_size(
            r#"query { list_object(x: [{ property: ["x", "y"] }, { property: ["x", "y"] }])} "#
                .to_string(),
            serde_json::json!({}),
        );
        assert_eq!(result.len(), 1);
        let first = result.first().unwrap();
        assert_eq!(first.size, 2);
        assert_eq!(first.name, "x");
    }

    #[test]
    fn list_object_size_variable() {
        let result = get_size(
            r#"query($x: [ObjectList]) { list_object(x: $x)} "#.to_string(),
            serde_json::json!({ "x": [{ "property": ["x", "y"] }, { "property": ["x", "y"] }] }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x");
    }

    #[test]
    fn list_nested_object_size() {
        let result = get_size(
            r#"query { list_object(x: [{ property: ["x", "y"] }])} "#.to_string(),
            serde_json::json!({}),
        );
        assert_eq!(result.len(), 1);
        let first = result.first().unwrap();
        assert_eq!(first.size, 2);
        assert_eq!(first.name, "x.0.property");
    }

    #[test]
    fn list_nested_object_size_variable() {
        let result = get_size(
            r#"query($x: [ObjectList]) { list_object(x: $x)} "#.to_string(),
            serde_json::json!({ "x": [{ "property": ["x", "y"] }] }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x.0.property");
    }
}
