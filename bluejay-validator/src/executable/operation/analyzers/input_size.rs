use crate::executable::{
    operation::{Analyzer, VariableValues, Visitor},
    Cache,
};
use bluejay_core::{
    definition::SchemaDefinition, executable::ExecutableDocument, Argument, AsIter, ObjectValue,
    Value, ValueReference, Variable,
};

#[derive(Clone)]
/// Represents an argument or input-field that exceeds
/// the maximum allowed list-size.
pub struct Offender {
    pub size: usize,
    pub name: String,
}

#[derive(Clone)]
/// The [InputSize] visitor will check all arguments and object-fields
/// for list-values, when it sees a list-value that exceeds the maximum
/// allowed list-size we will add it to the list of offenders.
/// As output we'll return an array of [Offender].
pub struct InputSize<'a, VV: VariableValues> {
    offenders: Vec<Offender>,
    max_length: usize,
    variable_values: &'a VV,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Visitor<'a, E, S, VV>
    for InputSize<'a, VV>
{
    type ExtraInfo = usize;

    fn new(
        _: &'a E::OperationDefinition,
        _s: &'a S,
        variables: &'a VV,
        _: &'a Cache<'a, E, S>,
        max_length: Self::ExtraInfo,
    ) -> Self {
        Self {
            max_length,
            offenders: vec![],
            variable_values: variables,
        }
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<false>,
        _input_value_definition: &'a S::InputValueDefinition,
    ) {
        find_input_size_offenders_arguments::<E, VV>(
            self.max_length,
            &mut self.offenders,
            self.variable_values,
            argument.name().to_string(),
            argument.value(),
        );
    }
}

/// Will go over all the arguments on a field and check whether the value is a list,
/// when it is a list it will check the input-size. When the input-size exceeds the maximum
/// allowed length we'll flag it as an offending argument, when it does not we'll traverse
/// deeper to find potential Objects contained within the list. When we enconter an object
/// we'll traverse deeper to find object-fields that contain lists as a value.
fn find_input_size_offenders_arguments<E: ExecutableDocument, VV: VariableValues>(
    max_length: usize,
    offenders: &mut Vec<Offender>,
    variable_values: &VV,
    argument_name: String,
    argument_value: &<E as bluejay_core::executable::ExecutableDocument>::Value<false>,
) {
    match argument_value.as_ref() {
        ValueReference::List(list) => {
            let list_length = list.len();
            if list_length > max_length {
                offenders.push(Offender {
                    size: list_length,
                    name: argument_name,
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
            let variable = variable_values.get(name);
            if let Some(value) = variable {
                find_input_size_offenders_variables::<E, VV>(
                    max_length,
                    offenders,
                    variable_values,
                    argument_name,
                    value,
                );
            }
        }
        _ => {}
    };
}

/// Similar to [find_input_size_offenders_arguments] however, it is specialised to traversing
/// variable-values.
fn find_input_size_offenders_variables<E: ExecutableDocument, VV: VariableValues>(
    max_length: usize,
    offenders: &mut Vec<Offender>,
    variable_values: &VV,
    argument_name: String,
    argument_value: &VV::Value,
) {
    match argument_value.as_ref() {
        ValueReference::List(list) => {
            let list_length = list.len();
            if list_length > max_length {
                offenders.push(Offender {
                    size: list_length,
                    name: argument_name,
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
        _ => {}
    };
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Analyzer<'a, E, S, VV>
    for InputSize<'a, VV>
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
        directive @test(y: [String]) on FIELD_DEFINITION

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

    fn analyze_input_size(query: &str, variables: serde_json::Value) -> Vec<Offender> {
        let definition_document: DefinitionDocument<'_, DefaultContext> =
            DefinitionDocument::parse(TEST_SCHEMA).expect("Schema had parse errors");
        let schema_definition =
            ParserSchemaDefinition::try_from(&definition_document).expect("Schema had errors");
        let executable_document = ParserExecutableDocument::parse(query)
            .unwrap_or_else(|_| panic!("Document had parse errors"));
        let cache = Cache::new(&executable_document, &schema_definition);
        let variables = variables.as_object().expect("Variables must be an object");
        Orchestrator::<_, _, JsonMap<String, JsonValue>, InputSize<_>>::analyze(
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
        let result =
            analyze_input_size(r#"query { simple(x: ["x", "y"])} "#, serde_json::json!({}));
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x");
    }

    #[test]
    fn simple_directive_size() {
        let result = analyze_input_size(
            r#"query { simple(x: []) @test(y: ["x", "y"])} "#,
            serde_json::json!({}),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "y");
    }

    #[test]
    fn simple_size_variable() {
        let result = analyze_input_size(
            r#"query ($x: [String]) { simple(x: $x)} "#,
            serde_json::json!({ "x": ["x", "y"] }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x");
    }

    #[test]
    fn object_size() {
        let result = analyze_input_size(
            r#"query { object(x: { property: ["x", "y"] })} "#,
            serde_json::json!({}),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x.property");
    }

    #[test]
    fn object_size_variable() {
        let result = analyze_input_size(
            r#"query($x: ObjectList) { object(x: $x)} "#,
            serde_json::json!({ "x": { "property": ["x", "y"] } }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x.property");
    }

    #[test]
    fn list_object_size() {
        let result = analyze_input_size(
            r#"query { list_object(x: [{ property: ["x", "y"] }, { property: ["x", "y"] }])} "#,
            serde_json::json!({}),
        );
        assert_eq!(result.len(), 1);
        let first = result.first().unwrap();
        assert_eq!(first.size, 2);
        assert_eq!(first.name, "x");
    }

    #[test]
    fn list_object_size_variable() {
        let result = analyze_input_size(
            r#"query($x: [ObjectList]) { list_object(x: $x)} "#,
            serde_json::json!({ "x": [{ "property": ["x", "y"] }, { "property": ["x", "y"] }] }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x");
    }

    #[test]
    fn list_nested_object_size() {
        let result = analyze_input_size(
            r#"query { list_object(x: [{ property: ["x", "y"] }])} "#,
            serde_json::json!({}),
        );
        assert_eq!(result.len(), 1);
        let first = result.first().unwrap();
        assert_eq!(first.size, 2);
        assert_eq!(first.name, "x.0.property");
    }

    #[test]
    fn list_nested_object_size_variable() {
        let result = analyze_input_size(
            r#"query($x: [ObjectList]) { list_object(x: $x)} "#,
            serde_json::json!({ "x": [{ "property": ["x", "y"] }] }),
        );
        let result = result.first().unwrap();
        assert_eq!(result.size, 2);
        assert_eq!(result.name, "x.0.property");
    }
}
