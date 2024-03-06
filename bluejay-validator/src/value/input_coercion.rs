use crate::Path;
use bluejay_core::definition::{
    BaseInputTypeReference, EnumTypeDefinition, EnumValueDefinition, InputFieldsDefinition,
    InputObjectTypeDefinition, InputType, InputTypeReference, InputValueDefinition,
    ScalarTypeDefinition, SchemaDefinition,
};
use bluejay_core::{
    AsIter, BuiltinScalarDefinition, Directive, ObjectValue, Value, ValueReference,
};
use std::collections::BTreeMap;

mod error;

pub use error::Error;

pub trait CoerceInput: SchemaDefinition {
    fn coerce_value<
        'a,
        const CONST: bool,
        I: InputType<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
        V: Value<CONST>,
    >(
        &'a self,
        input_type: &'a I,
        value: &'a V,
        path: Path<'a>,
    ) -> Result<(), Vec<Error<'a, CONST, V>>>;

    fn coerce_const_value<
        'a,
        I: InputType<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
        V: Value<true>,
    >(
        &'a self,
        input_type: &'a I,
        value: &'a V,
        path: Path<'a>,
    ) -> Result<(), Vec<Error<'a, true, V>>> {
        self.coerce_value(input_type, value, path)
    }
}

impl<S: SchemaDefinition> CoerceInput for S {
    fn coerce_value<
        'a,
        const CONST: bool,
        I: InputType<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
        V: Value<CONST>,
    >(
        &'a self,
        input_type: &'a I,
        value: &'a V,
        path: Path<'a>,
    ) -> Result<(), Vec<Error<'a, CONST, V>>> {
        coerce_value_for_input_type(self, input_type, value, path, true)
    }
}

fn coerce_value_for_input_type<
    'a,
    const CONST: bool,
    S: SchemaDefinition,
    T: InputType<
        CustomScalarTypeDefinition = S::CustomScalarTypeDefinition,
        InputObjectTypeDefinition = S::InputObjectTypeDefinition,
        EnumTypeDefinition = S::EnumTypeDefinition,
    >,
    V: Value<CONST>,
>(
    schema_definition: &'a S,
    input_type: &'a T,
    value: &'a V,
    path: Path<'a>,
    allow_implicit_list: bool,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    let core_type = input_type.as_ref(schema_definition);
    let is_required = core_type.is_required();
    match value.as_ref() {
        ValueReference::Null if is_required => Err(vec![Error::NullValueForRequiredType {
            value,
            input_type_name: input_type.display_name(),
            path,
        }]),
        ValueReference::Null | ValueReference::Variable(_) => Ok(()),
        core_value => match core_type {
            InputTypeReference::Base(_, _) => {
                coerce_value_for_base_input_type(schema_definition, input_type, value, path)
            }
            InputTypeReference::List(inner, _) => {
                if let ValueReference::List(values) = core_value {
                    let errors: Vec<Error<'a, CONST, V>> = values
                        .iter()
                        .enumerate()
                        .flat_map(|(idx, value)| {
                            coerce_value_for_input_type(
                                schema_definition,
                                inner,
                                value,
                                path.push(idx),
                                false,
                            )
                            .err()
                            .unwrap_or_default()
                        })
                        .collect();

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors)
                    }
                } else if allow_implicit_list {
                    coerce_value_for_input_type(schema_definition, inner, value, path, true)
                } else {
                    Err(vec![Error::NoImplicitConversion {
                        value,
                        input_type_name: input_type.display_name(),
                        path,
                    }])
                }
            }
        },
    }
}

fn coerce_value_for_base_input_type<
    'a,
    const CONST: bool,
    S: SchemaDefinition,
    T: InputType<
        CustomScalarTypeDefinition = S::CustomScalarTypeDefinition,
        InputObjectTypeDefinition = S::InputObjectTypeDefinition,
        EnumTypeDefinition = S::EnumTypeDefinition,
    >,
    V: Value<CONST>,
>(
    schema_definition: &'a S,
    input_type: &'a T,
    value: &'a V,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    let base = input_type.base(schema_definition);
    match base {
        BaseInputTypeReference::BuiltinScalar(bstd) => {
            coerce_builtin_scalar_value(input_type, bstd, value, path)
        }
        BaseInputTypeReference::CustomScalar(cstd) => coerce_custom_scalar_value(cstd, value, path),
        BaseInputTypeReference::Enum(etd) => coerce_enum_value(input_type, etd, value, path),
        BaseInputTypeReference::InputObject(iotd) => {
            coerce_input_object_value(schema_definition, input_type, iotd, value, path)
        }
    }
}

fn coerce_builtin_scalar_value<'a, const CONST: bool, V: Value<CONST>, T: InputType>(
    input_type: &'a T,
    bstd: BuiltinScalarDefinition,
    value: &'a V,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    match (bstd, value.as_ref()) {
        (BuiltinScalarDefinition::Boolean, ValueReference::Boolean(_)) => Ok(()),
        (BuiltinScalarDefinition::Float, ValueReference::Float(_)) => Ok(()),
        (BuiltinScalarDefinition::Float, ValueReference::Integer(_)) => Ok(()),
        (BuiltinScalarDefinition::ID, ValueReference::Integer(_)) => Ok(()),
        (
            BuiltinScalarDefinition::ID | BuiltinScalarDefinition::String,
            ValueReference::String(_),
        ) => Ok(()),
        (BuiltinScalarDefinition::Int, ValueReference::Integer(_)) => Ok(()),
        _ => Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type.display_name(),
            path,
        }]),
    }
}

fn coerce_custom_scalar_value<'a, const CONST: bool, V: Value<CONST>>(
    cstd: &'a impl ScalarTypeDefinition,
    value: &'a V,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    cstd.coerce_input(value).map_err(|message| {
        vec![Error::CustomScalarInvalidValue {
            value,
            custom_scalar_type_name: cstd.name(),
            message,
            path,
        }]
    })
}

fn coerce_enum_value<'a, const CONST: bool, V: Value<CONST>, T: InputType>(
    input_type: &'a T,
    enum_type_definition: &'a T::EnumTypeDefinition,
    value: &'a V,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    match value.as_ref() {
        ValueReference::Enum(name) => {
            coerce_enum_value_from_name(enum_type_definition, value, name, path)
        }
        ValueReference::String(name) if V::can_coerce_string_value_to_enum() => {
            coerce_enum_value_from_name(enum_type_definition, value, name, path)
        }
        _ => Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type.display_name(),
            path,
        }]),
    }
}

fn coerce_enum_value_from_name<'a, const CONST: bool, V: Value<CONST>>(
    enum_type_definition: &'a impl EnumTypeDefinition,
    value: &'a V,
    name: &'a str,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    if enum_type_definition
        .enum_value_definitions()
        .iter()
        .any(|evd| evd.name() == name)
    {
        Ok(())
    } else {
        Err(vec![Error::NoEnumMemberWithName {
            name,
            value,
            enum_type_name: enum_type_definition.name(),
            path,
        }])
    }
}

fn coerce_input_object_value<
    'a,
    const CONST: bool,
    S: SchemaDefinition,
    T: InputType<
        CustomScalarTypeDefinition = S::CustomScalarTypeDefinition,
        InputObjectTypeDefinition = S::InputObjectTypeDefinition,
        EnumTypeDefinition = S::EnumTypeDefinition,
    >,
    V: Value<CONST>,
>(
    schema_definition: &'a S,
    input_type: &'a T,
    input_object_type_definition: &'a T::InputObjectTypeDefinition,
    value: &'a V,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    if let ValueReference::Object(object) = value.as_ref() {
        let mut errors = Vec::new();
        let mut missing_required_values = Vec::new();

        type Entry<'a, const CONST: bool, V> = (
            &'a <<V as Value<CONST>>::Object as ObjectValue<CONST>>::Key,
            &'a V,
        );
        let indexed_object: BTreeMap<&'a str, Vec<Entry<'a, CONST, V>>> =
            object
                .iter()
                .fold(BTreeMap::new(), |mut index, (key, value)| {
                    index.entry(key.as_ref()).or_default().push((key, value));
                    index
                });

        errors.extend(
            indexed_object
                .iter()
                .filter(|(_, entries)| (entries.len() > 1))
                .map(|(&field_name, entries)| Error::NonUniqueFieldNames {
                    value,
                    field_name,
                    keys: Vec::from_iter(entries.iter().map(|&(key, _)| key)),
                    path: path.clone(),
                }),
        );

        errors.extend(
            object
                .iter()
                .filter(|(field, _)| {
                    input_object_type_definition
                        .input_field_definitions()
                        .get(field.as_ref())
                        .is_none()
                })
                .map(|(field, _)| Error::NoInputFieldWithName {
                    field,
                    input_object_type_name: input_object_type_definition.name(),
                    path: path.push(field.as_ref()),
                }),
        );

        input_object_type_definition
            .input_field_definitions()
            .iter()
            .for_each(|ivd| {
                let value_for_field = indexed_object
                    .get(ivd.name())
                    .and_then(|entries| entries.first().copied().map(|(_, value)| value));
                let default_value = ivd.default_value();

                match (value_for_field, default_value) {
                    (None, None) => {
                        if ivd.r#type().is_required() {
                            missing_required_values.push(ivd.name());
                        }
                    }
                    (None, Some(_)) => {}
                    (Some(value), _) => {
                        match schema_definition.coerce_value(
                            ivd.r#type(),
                            value,
                            path.push(ivd.name()),
                        ) {
                            Ok(_) => {}
                            Err(errs) => errors.extend(errs),
                        }
                    }
                }
            });

        #[cfg(feature = "one-of-input-objects")]
        if let Err(one_of_errors) = validate_one_of_input_object_value(
            input_object_type_definition,
            value,
            object,
            path.clone(),
        ) {
            errors.extend(one_of_errors);
        }

        if !missing_required_values.is_empty() {
            errors.push(Error::NoValueForRequiredFields {
                value,
                field_names: missing_required_values,
                input_object_type_name: input_object_type_definition.name(),
                path,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    } else {
        Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type.display_name(),
            path,
        }])
    }
}

#[cfg(feature = "one-of-input-objects")]
fn validate_one_of_input_object_value<'a, const CONST: bool, V: Value<CONST>>(
    input_object_type_definition: &'a impl InputObjectTypeDefinition,
    value: &'a V,
    object: &'a V::Object,
    path: Path<'a>,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    if input_object_type_definition
        .directives()
        .map(|directives| {
            directives
                .iter()
                .any(|directive| directive.name() == "oneOf")
        })
        .unwrap_or(false)
    {
        let (null_entries, non_null_entries): (Vec<_>, Vec<_>) = object
            .iter()
            .partition(|(_, value)| matches!(value.as_ref(), ValueReference::Null));

        let mut errors = Vec::new();

        if !null_entries.is_empty() {
            errors.push(Error::OneOfInputNullValues {
                value,
                input_object_type_name: input_object_type_definition.name(),
                null_entries,
                path: path.clone(),
            });
        }

        if non_null_entries.len() != 1 {
            errors.push(Error::OneOfInputNotSingleNonNullValue {
                value,
                input_object_type_name: input_object_type_definition.name(),
                non_null_entries,
                path,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CoerceInput, Error};
    use crate::Path;
    use bluejay_core::definition::{
        ArgumentsDefinition, FieldDefinition, FieldsDefinition, InputType, InputValueDefinition,
        ObjectTypeDefinition, ScalarTypeDefinition, SchemaDefinition,
    };
    use bluejay_core::{Value, ValueReference};
    use bluejay_parser::ast::{
        definition::{
            Context, CustomScalarTypeDefinition, DefinitionDocument, InputType as ParserInputType,
            SchemaDefinition as ParserSchemaDefinition,
        },
        Parse,
    };
    use once_cell::sync::Lazy;
    use serde_json::json;
    use std::borrow::Cow;

    #[derive(Debug)]
    struct CustomContext;

    impl Context for CustomContext {
        fn coerce_custom_scalar_input<const CONST: bool>(
            cstd: &CustomScalarTypeDefinition<Self>,
            value: &impl Value<CONST>,
        ) -> Result<(), Cow<'static, str>> {
            let value = value.as_ref();
            match cstd.name() {
                "Decimal" => {
                    if let ValueReference::String(s) = value {
                        s.parse::<f64>()
                            .map_err(|_| Cow::Owned(format!("Unable to parse `{s}` to Decimal")))
                            .and_then(|f| {
                                if f.is_finite() {
                                    Ok(())
                                } else {
                                    Err(Cow::Borrowed("Decimal values must be finite"))
                                }
                            })
                    } else {
                        Err(Cow::Owned(format!(
                            "Cannot coerce {} to Decimal",
                            value.variant()
                        )))
                    }
                }
                _ => Ok(()),
            }
        }
    }

    const SCHEMA: &str = r#"
    type Query {
      field(
        stringArg: String!
        intArg: Int!
        floatArg: Float!
        idArg: ID!
        booleanArg: Boolean!
        optionalArg: Int
        optionalListArg: [Int]
        optionalListOfListArg: [[Int]]
        enumArg: Choices!
        inputObjectArg: CustomInput!
        decimalArg: Decimal!
        oneOfInputObjectArg: InputUnion!
      ): Boolean!
    }

    enum Choices {
      FIRST
      SECOND
    }

    input CustomInput {
      stringArg: String!
      optionalStringArg: String
      stringArgWithDefault: String! = ""
    }

    scalar Decimal

    input InputUnion @oneOf {
        first: String
        second: Int
    }
    "#;

    static DEFINITION_DOCUMENT: Lazy<DefinitionDocument<'static, CustomContext>> =
        Lazy::new(|| DefinitionDocument::parse(SCHEMA).expect("Schema had parse errors"));
    static SCHEMA_DEFINITION: Lazy<ParserSchemaDefinition<'static, CustomContext>> =
        Lazy::new(|| {
            ParserSchemaDefinition::try_from(&*DEFINITION_DOCUMENT).expect("Schema had errors")
        });

    fn input_type(
        type_name: &str,
        field_name: &str,
        arg_name: &str,
    ) -> &'static ParserInputType<'static, CustomContext> {
        SCHEMA_DEFINITION
            .get_type_definition(type_name)
            .unwrap()
            .into_object()
            .unwrap()
            .fields_definition()
            .get(field_name)
            .unwrap()
            .arguments_definition()
            .unwrap()
            .get(arg_name)
            .unwrap()
            .r#type()
    }

    #[test]
    fn test_string() {
        let it = input_type("Query", "field", "stringArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(
                it,
                &json!("This is a string"),
                Default::default()
            )
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(123),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123), Default::default()),
        );
    }

    #[test]
    fn test_int() {
        let it = input_type("Query", "field", "intArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123), Default::default())
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(123.4),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123.4), Default::default()),
        );
    }

    #[test]
    fn test_float() {
        let it = input_type("Query", "field", "floatArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123.456), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123), Default::default())
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!("123.4"),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("123.4"), Default::default()),
        );
    }

    #[test]
    fn test_id() {
        let it = input_type("Query", "field", "idArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(1), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("a"), Default::default())
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(123.4),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123.4), Default::default()),
        );
    }

    #[test]
    fn test_boolean() {
        let it = input_type("Query", "field", "booleanArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(true), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(false), Default::default())
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(1),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(1), Default::default()),
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!("true"),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("true"), Default::default()),
        );
    }

    #[test]
    fn test_optional() {
        let it = input_type("Query", "field", "optionalArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(null), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123), Default::default())
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!("123"),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("123"), Default::default()),
        );
    }

    #[test]
    fn test_optional_list() {
        let it = input_type("Query", "field", "optionalListArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(null), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(1), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!([1]), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!([1, 2, 3]), Default::default())
        );
        assert_eq!(
            Err(vec![
                Error::NoImplicitConversion {
                    value: &json!("b"),
                    input_type_name: "Int".to_string(),
                    path: Path::new(1),
                },
                Error::NoImplicitConversion {
                    value: &json!(true),
                    input_type_name: "Int".to_string(),
                    path: Path::new(2),
                },
            ]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!([1, "b", true]), Default::default()),
        );
    }

    #[test]
    fn test_optional_list_of_list() {
        let it = input_type("Query", "field", "optionalListOfListArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(null), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(1), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!([[1], [2, 3]]), Default::default())
        );
        assert_eq!(
            Err(vec![
                Error::NoImplicitConversion {
                    value: &json!(1),
                    input_type_name: "[Int]".to_string(),
                    path: Path::new(0),
                },
                Error::NoImplicitConversion {
                    value: &json!(2),
                    input_type_name: "[Int]".to_string(),
                    path: Path::new(1),
                },
                Error::NoImplicitConversion {
                    value: &json!(3),
                    input_type_name: "[Int]".to_string(),
                    path: Path::new(2),
                },
            ]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!([1, 2, 3]), Default::default()),
        );
    }

    #[test]
    fn test_enum() {
        let it = input_type("Query", "field", "enumArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("FIRST"), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("SECOND"), Default::default())
        );
        assert_eq!(
            Err(vec![Error::NoEnumMemberWithName {
                name: "first",
                value: &json!("first"),
                enum_type_name: "Choices",
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("first"), Default::default()),
        );
    }

    #[test]
    fn test_input_object() {
        let it = input_type("Query", "field", "inputObjectArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(
                it,
                &json!({ "stringArg": "abc" }),
                Default::default()
            ),
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it,
                &json!({ "stringArg": "abc", "optionalStringArg": "def", "stringArgWithDefault": "ghi" }),
                Default::default(),
            ),
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(""),
                input_type_name: it.display_name(),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(""), Default::default()),
        );
        assert_eq!(
            Err(vec![Error::NoValueForRequiredFields {
                value: &json!({}),
                field_names: vec!["stringArg"],
                input_object_type_name: "CustomInput",
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!({}), Default::default()),
        );
        assert_eq!(
            Err(vec![Error::NoInputFieldWithName {
                field: &"notDefined".to_owned(),
                input_object_type_name: "CustomInput",
                path: Path::new("notDefined"),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(
                it,
                &json!({ "stringArg": "abc", "notDefined": "def" }),
                Default::default()
            ),
        );
        assert_eq!(
            Err(vec![Error::NullValueForRequiredType {
                value: &json!(null),
                input_type_name: "String!".to_owned(),
                path: Path::new("stringArgWithDefault"),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(
                it,
                &json!({ "stringArg": "abc", "stringArgWithDefault": null }),
                Default::default()
            ),
        );
    }

    #[test]
    fn test_custom_scalar() {
        let it = input_type("Query", "field", "decimalArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!("123.456"), Default::default())
        );
        assert_eq!(
            Err(vec![Error::CustomScalarInvalidValue {
                value: &json!(123.456),
                custom_scalar_type_name: "Decimal",
                message: Cow::Owned("Cannot coerce float to Decimal".to_owned()),
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!(123.456), Default::default()),
        );
    }

    #[test]
    fn test_one_of_input_object() {
        let it = input_type("Query", "field", "oneOfInputObjectArg");

        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!({ "first": "s" }), Default::default())
        );
        assert_eq!(
            Ok(()),
            SCHEMA_DEFINITION.coerce_const_value(it, &json!({ "second": 1 }), Default::default())
        );
        assert_eq!(
            Err(vec![Error::OneOfInputNullValues {
                value: &json!({ "first": null, "second": 1 }),
                input_object_type_name: "InputUnion",
                null_entries: vec![(&"first".to_owned(), &json!(null))],
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(
                it,
                &json!({ "first": null, "second": 1 }),
                Default::default()
            ),
        );
        assert_eq!(
            Err(vec![Error::OneOfInputNotSingleNonNullValue {
                value: &json!({ "first": "s", "second": 1 }),
                input_object_type_name: "InputUnion",
                non_null_entries: vec![
                    (&"first".to_owned(), &json!("s")),
                    (&"second".to_owned(), &json!(1))
                ],
                path: Default::default(),
            }]),
            SCHEMA_DEFINITION.coerce_const_value(
                it,
                &json!({ "first": "s", "second": 1 }),
                Default::default()
            ),
        )
    }
}
