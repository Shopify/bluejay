use crate::executable::{
    operation::{Analyzer, VariableValues, Visitor},
    Cache,
};
use bluejay_core::definition::{
    BaseInputTypeReference, EnumTypeDefinition, EnumValueDefinition, HasDirectives,
    InputObjectTypeDefinition, InputType, InputTypeReference, InputValueDefinition,
    SchemaDefinition,
};
use bluejay_core::executable::{ExecutableDocument, Field, VariableDefinition};
use bluejay_core::{Argument, AsIter, Directive, ObjectValue, Value, ValueReference};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// The deprecated usage we encountered.
pub enum UsageType {
    Argument,
    EnumValue,
    InputField,
    Field,
    Variable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Offender<'a> {
    pub reason: &'a str,
    pub offense_type: UsageType,
    pub name: &'a str,
}

/// The [Deprecation] analyzer will go over all ast-nodes of type Field, EnumValue, Argument and InputField
/// when it encounters one that is marked as deprecated while being used in the executable document
/// it will be added ot the list of [Offender].
/// This method will output the list of [Offender].
pub struct Deprecation<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> {
    offenders: Vec<Offender<'a>>,
    schema_definition: &'a S,
    cache: &'a Cache<'a, E, S>,
    variable_values: &'a VV,
}

const DEPRECATED_DIRECTIVE: &str = "deprecated";
const DEPRECATION_REASON: &str = "reason";

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Visitor<'a, E, S, VV>
    for Deprecation<'a, E, S, VV>
{
    type ExtraInfo = ();
    fn new(
        _: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a VV,
        cache: &'a Cache<'a, E, S>,
        _: Self::ExtraInfo,
    ) -> Self {
        Self {
            offenders: vec![],
            schema_definition,
            cache,
            variable_values,
        }
    }

    fn visit_variable_definition(
        &mut self,
        variable_definition: &'a <E as ExecutableDocument>::VariableDefinition,
    ) {
        if let Some(input_type) = self
            .cache
            .variable_definition_input_type(variable_definition.r#type())
        {
            if let Some(value) = self
                .variable_values
                .get(variable_definition.variable().as_ref())
            {
                self.find_deprecations_for_value(input_type, value, variable_definition.variable());
            }
            if let Some(default_value) = variable_definition.default_value() {
                self.find_deprecations_for_value(
                    input_type,
                    default_value,
                    variable_definition.variable(),
                );
            }
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
                name: field.name(),
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
                name: argument.name(),
                offense_type: UsageType::Argument,
                reason,
            });
        }

        self.find_deprecations_for_value(
            input_value_definition.r#type(),
            argument.value(),
            argument.name(),
        );
    }
}

fn get_deprecation_reason<N: HasDirectives>(ast_item: &N) -> Option<&str> {
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
    })
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Deprecation<'a, E, S, VV> {
    fn find_deprecations_for_value<
        const CONST: bool,
        I: InputType<
            CustomScalarTypeDefinition = S::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = S::InputObjectTypeDefinition,
            EnumTypeDefinition = S::EnumTypeDefinition,
        >,
        V: Value<CONST>,
    >(
        &mut self,
        input_type: &'a I,
        value: &'a V,
        name: &'a str,
    ) {
        match input_type.as_ref(self.schema_definition) {
            InputTypeReference::List(inner_list_type, _) => match value.as_ref() {
                ValueReference::List(list_value) => list_value.iter().for_each(|list_item| {
                    self.find_deprecations_for_value(inner_list_type, list_item, name)
                }),
                _ => self.find_deprecations_for_value(inner_list_type, value, name),
            },
            InputTypeReference::Base(base_input_type, _) => match base_input_type {
                BaseInputTypeReference::Enum(etd) => {
                    let enum_value = match value.as_ref() {
                        ValueReference::Enum(enum_value) => Some(enum_value),
                        ValueReference::String(string_value)
                            if V::can_coerce_string_value_to_enum() =>
                        {
                            Some(string_value)
                        }
                        _ => None,
                    };
                    if let Some(enum_value) = enum_value {
                        if let Some(deprecation_reason) = etd
                            .enum_value_definitions()
                            .iter()
                            .find(|evd| evd.name() == enum_value)
                            .and_then(|found_enum_value| {
                                get_deprecation_reason::<S::EnumValueDefinition>(found_enum_value)
                            })
                        {
                            self.offenders.push(Offender {
                                name,
                                offense_type: UsageType::EnumValue,
                                reason: deprecation_reason,
                            });
                        }
                    }
                }
                BaseInputTypeReference::InputObject(iotd) => {
                    if let ValueReference::Object(obj_value) = value.as_ref() {
                        iotd.input_field_definitions()
                            .iter()
                            .for_each(|input_field_definition| {
                                let found_usage = obj_value.iter().find(|(key, _value)| {
                                    key.as_ref() == input_field_definition.name()
                                });

                                if let Some((_, value)) = found_usage {
                                    if let Some(reason) =
                                        get_deprecation_reason::<S::InputValueDefinition>(
                                            input_field_definition,
                                        )
                                    {
                                        self.offenders.push(Offender {
                                            name: input_field_definition.name(),
                                            offense_type: UsageType::InputField,
                                            reason,
                                        });
                                    }

                                    self.find_deprecations_for_value(
                                        input_field_definition.r#type(),
                                        value,
                                        name,
                                    )
                                }
                            });
                    }
                }
                _ => {}
            },
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, VV: VariableValues> Analyzer<'a, E, S, VV>
    for Deprecation<'a, E, S, VV>
{
    type Output = Vec<Offender<'a>>;

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
        definition::{DefinitionDocument, SchemaDefinition as ParserSchemaDefinition},
        executable::ExecutableDocument as ParserExecutableDocument,
        Parse,
    };
    use once_cell::sync::Lazy;
    use serde_json::{Map as JsonMap, Value as JsonValue};

    type DeprecationAnalyzer<'a, E, S> = Orchestrator<
        'a,
        E,
        S,
        JsonMap<String, JsonValue>,
        Deprecation<'a, E, S, JsonMap<String, JsonValue>>,
    >;

    const TEST_SCHEMA_SDL: &str = r#"
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

    static TEST_DEFINITION_DOCUMENT: Lazy<DefinitionDocument<'static>> =
        Lazy::new(|| DefinitionDocument::parse(TEST_SCHEMA_SDL).unwrap());

    static TEST_SCHEMA_DEFINITION: Lazy<ParserSchemaDefinition<'static>> =
        Lazy::new(|| ParserSchemaDefinition::try_from(&*TEST_DEFINITION_DOCUMENT).unwrap());

    fn validate_deprecations(query: &str, variables: serde_json::Value, expected: Vec<Offender>) {
        let executable_document = ParserExecutableDocument::parse(query)
            .unwrap_or_else(|_| panic!("Document had parse errors"));
        let cache = Cache::new(&executable_document, &*TEST_SCHEMA_DEFINITION);
        let variables = variables.as_object().expect("Variables must be an object");
        let deprecations = DeprecationAnalyzer::analyze(
            &executable_document,
            &*TEST_SCHEMA_DEFINITION,
            None,
            variables,
            &cache,
            (),
        )
        .unwrap();
        assert_eq!(deprecations, expected);
    }

    #[test]
    fn field_deprecation() {
        validate_deprecations(
            r#"query { test_field }"#,
            serde_json::json!({}),
            vec![Offender {
                name: "test_field",
                reason: "field",
                offense_type: UsageType::Field,
            }],
        );
    }

    #[test]
    fn valid_field() {
        validate_deprecations(r#"query { valid_field }"#, serde_json::json!({}), vec![]);
    }

    #[test]
    fn variable_enum_value_deprecation() {
        validate_deprecations(
            r#"query ($test: TestEnum) { test_enum(deprecated_enum: $test) }"#,
            serde_json::json!({ "test": "DEPRECATED" }),
            vec![Offender {
                name: "test",
                reason: "enum_value",
                offense_type: UsageType::EnumValue,
            }],
        );
    }

    #[test]
    fn enum_value_deprecation() {
        validate_deprecations(
            r#"query { test_enum(deprecated_enum: DEPRECATED) }"#,
            serde_json::json!({}),
            vec![Offender {
                name: "deprecated_enum",
                reason: "enum_value",
                offense_type: UsageType::EnumValue,
            }],
        );
    }

    #[test]
    fn arg_deprecation() {
        validate_deprecations(
            r#"query { test_arg(deprecated_arg: "x") }"#,
            serde_json::json!({}),
            vec![Offender {
                name: "deprecated_arg",
                reason: "arg",
                offense_type: UsageType::Argument,
            }],
        );
    }

    #[test]
    fn variable_arg_deprecation() {
        validate_deprecations(
            r#"query($test: String) { test_arg(deprecated_arg: $test) }"#,
            serde_json::json!({ "test": "x" }),
            vec![Offender {
                name: "deprecated_arg",
                reason: "arg",
                offense_type: UsageType::Argument,
            }],
        );
    }

    #[test]
    fn input_field_deprecation() {
        validate_deprecations(
            r#"query { test_input(input: { deprecated_input_field: "x" }) }"#,
            serde_json::json!({}),
            vec![Offender {
                name: "deprecated_input_field",
                reason: "input_field",
                offense_type: UsageType::InputField,
            }],
        );
    }

    #[test]
    fn variable_input_field_deprecation() {
        validate_deprecations(
            r#"query($input: TestInput) { test_input(input: $input) }"#,
            serde_json::json!({ "input": { "deprecated_input_field": "x" } }),
            vec![Offender {
                name: "deprecated_input_field",
                reason: "input_field",
                offense_type: UsageType::InputField,
            }],
        );
    }

    #[test]
    fn nested_variable_input_field_deprecation() {
        validate_deprecations(
            r#"query($test: String) { test_input(input: { deprecated_input_field: $test }) }"#,
            serde_json::json!({
                "test": "x"
            }),
            vec![Offender {
                name: "deprecated_input_field",
                reason: "input_field",
                offense_type: UsageType::InputField,
            }],
        );
    }

    #[test]
    fn nested_input_field_deprecation() {
        validate_deprecations(
            r#"query { test_nested_input(nested_input: { nested: { deprecated_input_field: "x" } }) }"#,
            serde_json::json!({}),
            vec![Offender {
                name: "deprecated_input_field",
                reason: "input_field",
                offense_type: UsageType::InputField,
            }],
        );
    }

    #[test]
    fn nested_list_input_field_deprecation() {
        validate_deprecations(
            r#"query { test_nested_input_list(nested_input: [{ nested: { deprecated_input_field: "x" } }]) }"#,
            serde_json::json!({}),
            vec![Offender {
                name: "deprecated_input_field",
                reason: "input_field",
                offense_type: UsageType::InputField,
            }],
        );
    }

    #[test]
    fn nested_variable_list_input_field_deprecation() {
        validate_deprecations(
            r#"query($test: [NestedInput]) { test_nested_input_list(nested_input: $test) }"#,
            serde_json::json!({ "test": [{ "nested": { "deprecated_input_field": "x" } }] }),
            vec![Offender {
                name: "deprecated_input_field",
                reason: "input_field",
                offense_type: UsageType::InputField,
            }],
        );
    }
}
