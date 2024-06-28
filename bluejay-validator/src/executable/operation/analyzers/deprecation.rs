use std::collections::HashMap;

use crate::executable::{
    operation::{Analyzer, VariableValues, Visitor},
    Cache,
};
use bluejay_core::definition::{
    BaseInputTypeReference, EnumTypeDefinition, InputType, InputTypeReference, InputValueDefinition,
};
use bluejay_core::definition::{EnumValueDefinition, InputObjectTypeDefinition};
use bluejay_core::executable::Field;
use bluejay_core::ObjectValue;
use bluejay_core::{definition::HasDirectives, Value};
use bluejay_core::{
    definition::SchemaDefinition, executable::ExecutableDocument, Argument, AsIter, Directive,
    ValueReference, Variable,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// The deprecated usage we encountered.
pub enum UsageType {
    Argument,
    EnumValue,
    InputField,
    Field,
}

#[derive(Clone, Debug)]
pub struct Offender {
    pub reason: String,
    pub offense_type: UsageType,
    pub name: String,
}

#[derive(Clone, Debug)]
/// The [Deprecation] analyzer will go over all ast-nodes of type Field, EnumValue, Argument and InputField
/// when it encounters one that is marked as deprecated while being used in the executable document
/// it will be added ot the list of [Offender].
/// This method will output the list of [Offender].
pub struct Deprecation<'a, S: SchemaDefinition, VV: VariableValues> {
    offenders: Vec<Offender>,
    schema_definition: &'a S,
    variable_values: HashMap<&'a str, (&'a VV::Key, &'a VV::Value)>,
}

const DEPRECATED_DIRECTIVE: &str = "deprecated";
const DEPRECATION_REASON: &str = "reason";

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Visitor<'a, E, S, VV>
    for Deprecation<'a, S, VV>
{
    type ExtraInfo = ();
    fn new(
        _: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variables: &'a VV,
        _: &'a Cache<'a, E, S>,
        _: Self::ExtraInfo,
    ) -> Self {
        Self {
            offenders: vec![],
            schema_definition,
            variable_values: variables
                .iter()
                .map(|(key, value)| (key.as_ref(), (key, value)))
                .collect(),
        }
    }

    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a <S as SchemaDefinition>::FieldDefinition,
        _scoped_type: bluejay_core::definition::TypeDefinitionReference<
            'a,
            <S as SchemaDefinition>::TypeDefinition,
        >,
        included: bool,
    ) {
        if !included {
            return;
        }

        if let Some(reason) =
            get_deprecation_reason::<<S as SchemaDefinition>::FieldDefinition>(field_definition)
        {
            self.offenders.push(Offender {
                name: field.name().to_string(),
                offense_type: UsageType::Field,
                reason,
            });
        }
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<false>,
        input_value_definition: &'a <S as SchemaDefinition>::InputValueDefinition,
    ) {
        if let Some(reason) = get_deprecation_reason::<<S as SchemaDefinition>::InputValueDefinition>(
            input_value_definition,
        ) {
            self.offenders.push(Offender {
                name: argument.name().to_string(),
                offense_type: UsageType::Argument,
                reason,
            });
        }

        find_deprecations_in_arguments::<E, S, VV>(
            input_value_definition.r#type(),
            argument.name(),
            argument.value(),
            self.schema_definition,
            &mut self.offenders,
            &self.variable_values,
        );
    }
}

fn get_deprecation_reason<N: HasDirectives>(ast_item: &N) -> Option<String> {
    let deprecated_directive = ast_item.directives().and_then(|directives| {
        directives
            .iter()
            .find(|directive| directive.name() == DEPRECATED_DIRECTIVE)
    });

    deprecated_directive.map(|deprecated_directive| {
        deprecated_directive
            .arguments()
            .and_then(|arguments| {
                arguments
                    .iter()
                    .find(|argument| argument.name() == DEPRECATION_REASON)
                    .and_then(|argument| {
                        if let ValueReference::String(str) = argument.value().as_ref() {
                            Some(str)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or("No longer supported.")
            .to_string()
    })
}

/// This function will go through the value of an argument to find:
///
/// - deprecated enum-values
/// - deprecated object-fields
///
/// To achieve this we need to traverse lists and objects and look at the values
/// they are using. When we encounter a deprecated input-field or enum-value we
/// need to ensure that the user is actually using this field/value.
fn find_deprecations_in_arguments<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    VV: VariableValues,
>(
    input_type: &'a <S as bluejay_core::definition::SchemaDefinition>::InputType,
    argument_name: &'a str,
    argument_value: &<E as bluejay_core::executable::ExecutableDocument>::Value<false>,
    schema_definition: &'a S,
    offenders: &mut Vec<Offender>,
    variable_values: &HashMap<&'a str, (&'a VV::Key, &'a VV::Value)>,
) {
    match input_type.as_ref(schema_definition) {
        InputTypeReference::List(inner_list_type, _) => {
            match argument_value.as_ref() {
                ValueReference::List(list_value) => list_value.iter().for_each(|list_item| {
                    find_deprecations_in_arguments::<E, S, VV>(
                        inner_list_type,
                        argument_name,
                        list_item,
                        schema_definition,
                        offenders,
                        variable_values,
                    );
                }),
                ValueReference::Variable(var) => {
                    let var = variable_values.get(var.name());
                    if let Some((_, variable_value)) = var {
                        if let ValueReference::List(list_value) = variable_value.as_ref() {
                            list_value.iter().for_each(|list_item| {
                                find_deprecations_in_variables::<E, S, VV>(
                                    inner_list_type,
                                    argument_name,
                                    list_item,
                                    schema_definition,
                                    offenders,
                                );
                            })
                        }
                    }
                }
                _ => {}
            };
        }
        InputTypeReference::Base(BaseInputTypeReference::InputObject(schema_obj), _) => {
            match argument_value.as_ref() {
                ValueReference::Object(obj_value) => {
                    schema_obj.input_field_definitions().iter().for_each(
                        |input_field_definition| {
                            let found_usage = obj_value.iter().find(|(key, _value)| {
                                key.as_ref() == input_field_definition.name()
                            });

                            if let Some(field) = found_usage {
                                if let Some(reason) =
                                    get_deprecation_reason::<S::InputValueDefinition>(
                                        input_field_definition,
                                    )
                                {
                                    offenders.push(Offender {
                                        name: input_field_definition.name().to_string(),
                                        offense_type: UsageType::InputField,
                                        reason,
                                    });
                                }

                                find_deprecations_in_arguments::<E, S, VV>(
                                    input_field_definition.r#type(),
                                    argument_name,
                                    field.1,
                                    schema_definition,
                                    offenders,
                                    variable_values,
                                )
                            }
                        },
                    );
                }
                ValueReference::Variable(var) => {
                    let var = variable_values.get(var.name());
                    if let Some((_, variable_value)) = var {
                        let obj_value = match variable_value.as_ref() {
                            ValueReference::Object(object_value) => Some(object_value),
                            _ => None,
                        };

                        if let Some(obj_value) = obj_value {
                            schema_obj.input_field_definitions().iter().for_each(
                                |input_field_definition| {
                                    let found_usage = obj_value.iter().find(|item| {
                                        item.0.as_ref() == input_field_definition.name()
                                    });

                                    if let Some(field) = found_usage {
                                        if let Some(reason) =
                                            get_deprecation_reason::<S::InputValueDefinition>(
                                                input_field_definition,
                                            )
                                        {
                                            offenders.push(Offender {
                                                name: input_field_definition.name().to_string(),
                                                offense_type: UsageType::InputField,
                                                reason,
                                            });
                                        }

                                        find_deprecations_in_variables::<E, S, VV>(
                                            input_field_definition.r#type(),
                                            argument_name,
                                            field.1,
                                            schema_definition,
                                            offenders,
                                        )
                                    }
                                },
                            );
                        }
                    }
                }
                _ => {}
            };
        }
        InputTypeReference::Base(BaseInputTypeReference::Enum(schema_enum), _) => {
            let enum_value = match argument_value.as_ref() {
                ValueReference::Enum(enum_value) => Some(enum_value),
                ValueReference::Variable(var) => {
                    let var = variable_values.get(var.name());
                    if let Some((_, variable_value)) = var {
                        match variable_value.as_ref() {
                            ValueReference::Enum(enum_value) => Some(enum_value),
                            ValueReference::String(string_value) => Some(string_value),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(enum_value) = enum_value {
                if let Some(deprecation_reason) = schema_enum
                    .enum_value_definitions()
                    .iter()
                    .find(|schema_enum_value| schema_enum_value.name() == enum_value)
                    .and_then(|found_enum_value| {
                        get_deprecation_reason::<S::EnumValueDefinition>(found_enum_value)
                    })
                {
                    offenders.push(Offender {
                        name: argument_name.to_string(),
                        offense_type: UsageType::EnumValue,
                        reason: deprecation_reason,
                    });
                }
            }
        }
        _ => {}
    };
}

fn find_deprecations_in_variables<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    VV: VariableValues,
>(
    input_type: &'a <S as bluejay_core::definition::SchemaDefinition>::InputType,
    argument_name: &'a str,
    argument_value: &VV::Value,
    schema_definition: &'a S,
    offenders: &mut Vec<Offender>,
) {
    match input_type.as_ref(schema_definition) {
        InputTypeReference::List(inner_list_type, _) => {
            if let ValueReference::List(list_value) = argument_value.as_ref() {
                list_value.iter().for_each(|list_item| {
                    find_deprecations_in_variables::<E, S, VV>(
                        inner_list_type,
                        argument_name,
                        list_item,
                        schema_definition,
                        offenders,
                    );
                })
            }
        }
        InputTypeReference::Base(BaseInputTypeReference::InputObject(schema_obj), _) => {
            let obj_value = match argument_value.as_ref() {
                ValueReference::Object(obj_value) => Some(obj_value),
                _ => None,
            };

            if let Some(obj_value) = obj_value {
                schema_obj
                    .input_field_definitions()
                    .iter()
                    .for_each(|input_field_definition| {
                        let found_usage = obj_value
                            .iter()
                            .find(|(key, _value)| key.as_ref() == input_field_definition.name());

                        if let Some(field) = found_usage {
                            if let Some(reason) = get_deprecation_reason::<S::InputValueDefinition>(
                                input_field_definition,
                            ) {
                                offenders.push(Offender {
                                    name: input_field_definition.name().to_string(),
                                    offense_type: UsageType::InputField,
                                    reason,
                                });
                            }

                            find_deprecations_in_variables::<E, S, VV>(
                                input_field_definition.r#type(),
                                argument_name,
                                field.1,
                                schema_definition,
                                offenders,
                            )
                        }
                    });
            }
        }
        InputTypeReference::Base(BaseInputTypeReference::Enum(schema_enum), _) => {
            if let ValueReference::Enum(enum_value) = argument_value.as_ref() {
                if let Some(deprecation_reason) = schema_enum
                    .enum_value_definitions()
                    .iter()
                    .find(|schema_enum_value| schema_enum_value.name() == enum_value)
                    .and_then(|found_enum_value| {
                        get_deprecation_reason::<S::EnumValueDefinition>(found_enum_value)
                    })
                {
                    offenders.push(Offender {
                        name: argument_name.to_string(),
                        offense_type: UsageType::EnumValue,
                        reason: deprecation_reason,
                    });
                }
            }
        }
        _ => {}
    };
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Analyzer<'a, E, S, VV>
    for Deprecation<'a, S, VV>
{
    type Output = Vec<Offender>;

    fn into_output(self) -> Self::Output {
        self.offenders
    }
}

#[cfg(test)]
mod tests {
    use super::{Deprecation, Offender};
    use crate::executable::{
        operation::{analyzers::deprecation::UsageType, Orchestrator},
        Cache,
    };
    use bluejay_parser::ast::{
        definition::{
            DefaultContext, DefinitionDocument, SchemaDefinition as ParserSchemaDefinition,
        },
        executable::ExecutableDocument as ParserExecutableDocument,
        Parse,
    };
    use serde_json::{Map as JsonMap, Value as JsonValue};

    type DeprecationAnalyzer<'a, E, S> = Orchestrator<
        'a,
        E,
        S,
        JsonMap<String, JsonValue>,
        Deprecation<'a, S, JsonMap<String, JsonValue>>,
    >;

    const TEST_SCHEMA: &str = r#"
        enum TestEnum {
            DEPRECATED @deprecated(reason: "enum_value")
        }

        input TestInput {
            deprecated_input_field: String @deprecated(reason: "input_field")
        }

        input NestedInput {
            nested: TestInput
        }

        type Query {
          valid_field: String!
          test_field: String! @deprecated(reason: "field")
          test_enum(deprecated_enum: TestEnum): String!
          test_arg(
            deprecated_arg: String @deprecated(reason: "arg")
          ): String!
          test_input(
            input: TestInput
          ): String!
          test_nested_input(nested_input: NestedInput): String!
          test_nested_input_list(nested_input: [NestedInput]): String!
        }
        schema {
          query: Query
        }
    "#;

    fn find_deprecations(query: String, variables: serde_json::Value) -> Vec<Offender> {
        let definition_document: DefinitionDocument<'_, DefaultContext> =
            DefinitionDocument::parse(TEST_SCHEMA).expect("Schema had parse errors");
        let schema_definition =
            ParserSchemaDefinition::try_from(&definition_document).expect("Schema had errors");
        let executable_document = ParserExecutableDocument::parse(&query)
            .unwrap_or_else(|_| panic!("Document had parse errors"));
        let cache = Cache::new(&executable_document, &schema_definition);
        let variables = variables.as_object().expect("Variables must be an object");
        DeprecationAnalyzer::analyze(
            &executable_document,
            &schema_definition,
            None,
            variables,
            &cache,
            (),
        )
        .unwrap()
    }

    #[test]
    fn field_deprecation() {
        let result =
            find_deprecations(r#"query { test_field }"#.to_string(), serde_json::json!({}));
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "test_field");
        assert_eq!(first_item.reason, "field");
        assert_eq!(first_item.offense_type, UsageType::Field);
    }

    #[test]
    fn valid_field() {
        let result = find_deprecations(
            r#"query { valid_field }"#.to_string(),
            serde_json::json!({}),
        );
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn variable_enum_value_deprecation() {
        let result = find_deprecations(
            r#"query ($test: TestEnum) { test_enum(deprecated_enum: $test) }"#.to_string(),
            serde_json::json!({ "test": "DEPRECATED" }),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_enum");
        assert_eq!(first_item.reason, "enum_value");
        assert_eq!(first_item.offense_type, UsageType::EnumValue);
    }

    #[test]
    fn enum_value_deprecation() {
        let result = find_deprecations(
            r#"query { test_enum(deprecated_enum: DEPRECATED) }"#.to_string(),
            serde_json::json!({}),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_enum");
        assert_eq!(first_item.reason, "enum_value");
        assert_eq!(first_item.offense_type, UsageType::EnumValue);
    }

    #[test]
    fn arg_deprecation() {
        let result = find_deprecations(
            r#"query { test_arg(deprecated_arg: "x") }"#.to_string(),
            serde_json::json!({}),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_arg");
        assert_eq!(first_item.reason, "arg");
        assert_eq!(first_item.offense_type, UsageType::Argument);
    }

    #[test]
    fn variable_arg_deprecation() {
        let result = find_deprecations(
            r#"query($test: String) { test_arg(deprecated_arg: $test) }"#.to_string(),
            serde_json::json!({ "test": "x" }),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_arg");
        assert_eq!(first_item.reason, "arg");
        assert_eq!(first_item.offense_type, UsageType::Argument);
    }

    #[test]
    fn input_field_deprecation() {
        let result = find_deprecations(
            r#"query { test_input(input: { deprecated_input_field: "x" }) }"#.to_string(),
            serde_json::json!({}),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_input_field");
        assert_eq!(first_item.reason, "input_field");
        assert_eq!(first_item.offense_type, UsageType::InputField);
    }

    #[test]
    fn variable_input_field_deprecation() {
        let result = find_deprecations(
            r#"query($input: TestInput) { test_input(input: $input) }"#.to_string(),
            serde_json::json!({ "input": { "deprecated_input_field": "x" } }),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_input_field");
        assert_eq!(first_item.reason, "input_field");
        assert_eq!(first_item.offense_type, UsageType::InputField);
    }

    #[test]
    fn nested_variable_input_field_deprecation() {
        let result = find_deprecations(
            r#"query($test: String) { test_input(input: { deprecated_input_field: $test }) }"#
                .to_string(),
            serde_json::json!({
                "test": "x"
            }),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_input_field");
        assert_eq!(first_item.reason, "input_field");
        assert_eq!(first_item.offense_type, UsageType::InputField);
    }

    #[test]
    fn nested_input_field_deprecation() {
        let result = find_deprecations(r#"query { test_nested_input(nested_input: { nested: { deprecated_input_field: "x" } }) }"#.to_string(), serde_json::json!({}));
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_input_field");
        assert_eq!(first_item.reason, "input_field");
        assert_eq!(first_item.offense_type, UsageType::InputField);
    }

    #[test]
    fn nested_list_input_field_deprecation() {
        let result = find_deprecations(r#"query { test_nested_input_list(nested_input: [{ nested: { deprecated_input_field: "x" } }]) }"#.to_string(), serde_json::json!({}));
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_input_field");
        assert_eq!(first_item.reason, "input_field");
        assert_eq!(first_item.offense_type, UsageType::InputField);
    }

    #[test]
    fn nested_variable_list_input_field_deprecation() {
        let result = find_deprecations(
            r#"query($test: [NestedInput]) { test_nested_input_list(nested_input: $test) }"#
                .to_string(),
            serde_json::json!({ "test": [{ "nested": { "deprecated_input_field": "x" } }] }),
        );
        let first_item = result.first().unwrap();
        assert_eq!(first_item.name, "deprecated_input_field");
        assert_eq!(first_item.reason, "input_field");
        assert_eq!(first_item.offense_type, UsageType::InputField);
    }
}
