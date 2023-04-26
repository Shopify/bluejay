use bluejay_core::definition::{
    AbstractBaseInputTypeReference, AbstractInputTypeReference, BaseInputTypeReference,
    EnumTypeDefinition, EnumValueDefinition, InputFieldsDefinition, InputObjectTypeDefinition,
    InputTypeReference, InputValueDefinition, ScalarTypeDefinition,
};
use bluejay_core::{AbstractValue, AsIter, BuiltinScalarDefinition, ObjectValue, Value};
use std::collections::BTreeMap;

mod error;

pub use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathMember<'a> {
    Key(&'a str),
    Index(usize),
}

pub trait CoerceInput: AbstractInputTypeReference {
    fn coerce_value<'a, const CONST: bool, V: AbstractValue<CONST>>(
        &'a self,
        value: &'a V,
        path: &[PathMember<'a>],
    ) -> Result<(), Vec<Error<'a, CONST, V>>>;

    fn coerce_const_value<'a, V: AbstractValue<true>>(
        &'a self,
        value: &'a V,
        path: &[PathMember<'a>],
    ) -> Result<(), Vec<Error<'a, true, V>>> {
        self.coerce_value(value, path)
    }
}

impl<T: AbstractInputTypeReference> CoerceInput for T {
    fn coerce_value<'a, const CONST: bool, V: AbstractValue<CONST>>(
        &'a self,
        value: &'a V,
        path: &[PathMember<'a>],
    ) -> Result<(), Vec<Error<'a, CONST, V>>> {
        coerce_value_for_input_type_reference(self, value, path, true)
    }
}

fn coerce_value_for_input_type_reference<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    value: &'a V,
    path: &[PathMember<'a>],
    allow_implicit_list: bool,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    let core_type = input_type_reference.as_ref();
    let is_required = core_type.is_required();
    match value.as_ref() {
        Value::Null if is_required => Err(vec![Error::NullValueForRequiredType {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }]),
        Value::Null | Value::Variable(_) => Ok(()),
        core_value => match core_type {
            InputTypeReference::Base(_, _) => {
                coerce_value_for_base_input_type_reference(input_type_reference, value, path)
            }
            InputTypeReference::List(inner, _) => {
                if let Value::List(values) = core_value {
                    let errors: Vec<Error<'a, CONST, V>> = values
                        .iter()
                        .enumerate()
                        .flat_map(|(idx, value)| {
                            let mut path = path.to_owned();
                            path.push(PathMember::Index(idx));
                            coerce_value_for_input_type_reference(inner, value, &path, false)
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
                    coerce_value_for_input_type_reference(inner, value, path, true)
                } else {
                    Err(vec![Error::NoImplicitConversion {
                        value,
                        input_type_name: input_type_reference.as_ref().display_name(),
                        path: path.to_owned(),
                    }])
                }
            }
        },
    }
}

fn coerce_value_for_base_input_type_reference<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    let base = input_type_reference.as_ref().base().as_ref();
    match base {
        BaseInputTypeReference::BuiltinScalarType(bstd) => {
            coerce_builtin_scalar_value(input_type_reference, bstd, value, path)
        }
        BaseInputTypeReference::CustomScalarType(cstd) => {
            coerce_custom_scalar_value(cstd, value, path)
        }
        BaseInputTypeReference::EnumType(etd) => {
            coerce_enum_value(input_type_reference, etd, value, path)
        }
        BaseInputTypeReference::InputObjectType(iotd) => {
            coerce_input_object_value(input_type_reference, iotd, value, path)
        }
    }
}

fn coerce_builtin_scalar_value<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    bstd: BuiltinScalarDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    match (bstd, value.as_ref()) {
        (BuiltinScalarDefinition::Boolean, Value::Boolean(_)) => Ok(()),
        (BuiltinScalarDefinition::Float, Value::Float(_)) => Ok(()),
        (BuiltinScalarDefinition::Float, Value::Integer(_)) => Ok(()),
        (BuiltinScalarDefinition::ID, Value::Integer(_)) => Ok(()),
        (BuiltinScalarDefinition::ID | BuiltinScalarDefinition::String, Value::String(_)) => Ok(()),
        (BuiltinScalarDefinition::Int, Value::Integer(_)) => Ok(()),
        _ => Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }]),
    }
}

fn coerce_custom_scalar_value<'a, const CONST: bool, V: AbstractValue<CONST>>(
    cstd: &'a impl ScalarTypeDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    cstd.coerce_input(value).map_err(|message| {
        vec![Error::CustomScalarInvalidValue {
            value,
            custom_scalar_type_name: cstd.name(),
            message,
            path: path.to_owned(),
        }]
    })
}

fn coerce_enum_value<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    enum_type_definition: &'a <T::BaseInputTypeReference as AbstractBaseInputTypeReference>::EnumTypeDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    match value.as_ref() {
        Value::Enum(name) => coerce_enum_value_from_name(enum_type_definition, value, name, path),
        Value::String(name) if V::can_coerce_string_value_to_enum() => {
            coerce_enum_value_from_name(enum_type_definition, value, name, path)
        }
        _ => Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }]),
    }
}

fn coerce_enum_value_from_name<'a, const CONST: bool, V: AbstractValue<CONST>>(
    enum_type_definition: &'a impl EnumTypeDefinition,
    value: &'a V,
    name: &'a str,
    path: &[PathMember<'a>],
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
            path: path.to_owned(),
        }])
    }
}

fn coerce_input_object_value<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    input_object_type_definition: &'a <T::BaseInputTypeReference as AbstractBaseInputTypeReference>::InputObjectTypeDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    if let Value::Object(object) = value.as_ref() {
        let mut errors = Vec::new();
        let mut missing_required_values = Vec::new();

        type Entry<'a, const CONST: bool, V> = (
            &'a <<V as AbstractValue<CONST>>::Object as ObjectValue<CONST>>::Key,
            &'a V,
        );
        let indexed_object: BTreeMap<&'a str, Vec<Entry<'a, CONST, V>>> =
            object
                .iter()
                .fold(BTreeMap::new(), |mut index, (key, value)| {
                    index.entry(key.as_ref()).or_default().push((key, value));
                    index
                });

        errors.extend(indexed_object.iter().filter_map(|(&field_name, entries)| {
            (entries.len() > 1).then(|| Error::NonUniqueFieldNames {
                value,
                field_name,
                keys: Vec::from_iter(entries.iter().map(|&(key, _)| key)),
                path: path.to_owned(),
            })
        }));

        errors.extend(object.iter().filter_map(|(field, _)| {
            input_object_type_definition
                .input_field_definitions()
                .get(field.as_ref())
                .is_none()
                .then(|| Error::NoInputFieldWithName {
                    field,
                    input_object_type_name: input_object_type_definition.name(),
                    path: {
                        let mut nested_path = path.to_owned();
                        nested_path.push(PathMember::Key(field.as_ref()));
                        nested_path
                    },
                })
        }));

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
                        if ivd.r#type().as_ref().is_required() {
                            missing_required_values.push(ivd.name());
                        }
                    }
                    (None, Some(_)) => {}
                    (Some(value), _) => {
                        let mut inner_path = path.to_owned();
                        inner_path.push(PathMember::Key(ivd.name()));
                        match ivd.r#type().coerce_value(value, &inner_path) {
                            Ok(_) => {}
                            Err(errs) => errors.extend(errs),
                        }
                    }
                }
            });

        if !missing_required_values.is_empty() {
            errors.push(Error::NoValueForRequiredFields {
                value,
                field_names: missing_required_values,
                input_object_type_name: input_object_type_definition.name(),
                path: path.to_owned(),
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
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::{CoerceInput, Error, PathMember};
    use bluejay_core::definition::{
        AbstractInputTypeReference, ArgumentsDefinition, FieldDefinition, FieldsDefinition,
        InputValueDefinition, ObjectTypeDefinition, ScalarTypeDefinition, SchemaDefinition,
    };
    use bluejay_core::{AbstractValue, Value};
    use bluejay_parser::ast::definition::{
        Context, CustomScalarTypeDefinition, DefinitionDocument,
        InputTypeReference as ParserInputTypeReference, SchemaDefinition as ParserSchemaDefinition,
    };
    use once_cell::sync::Lazy;
    use serde_json::json;
    use std::borrow::Cow;

    #[derive(Debug)]
    struct CustomContext;

    impl Context for CustomContext {
        fn coerce_custom_scalar_input<const CONST: bool>(
            cstd: &CustomScalarTypeDefinition<Self>,
            value: &impl AbstractValue<CONST>,
        ) -> Result<(), Cow<'static, str>> {
            let value = value.as_ref();
            match cstd.name() {
                "Decimal" => {
                    if let Value::String(s) = value {
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
                        Err(Cow::Owned(format!("Cannot coerce {value} to Decimal")))
                    }
                }
                _ => Ok(()),
            }
        }
    }

    const SCHEMA: &'static str = r#"
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
    "#;

    static DEFINITION_DOCUMENT: Lazy<DefinitionDocument<'static, CustomContext>> =
        Lazy::new(|| DefinitionDocument::parse(SCHEMA).expect("Schema had parse errors"));
    static SCHEMA_DEFINITION: Lazy<ParserSchemaDefinition<'static, CustomContext>> =
        Lazy::new(|| {
            ParserSchemaDefinition::try_from(&*DEFINITION_DOCUMENT).expect("Schema had errors")
        });

    fn input_type_reference(
        type_name: &str,
        field_name: &str,
        arg_name: &str,
    ) -> &'static ParserInputTypeReference<'static, CustomContext> {
        SCHEMA_DEFINITION
            .get_type_definition(type_name)
            .unwrap()
            .into_object_type()
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
        let itr = input_type_reference("Query", "field", "stringArg");

        assert_eq!(
            Ok(()),
            itr.coerce_const_value(&json!("This is a string"), &[])
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(123),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!(123), &[]),
        );
    }

    #[test]
    fn test_int() {
        let itr = input_type_reference("Query", "field", "intArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(123), &[]));
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(123.4),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!(123.4), &[]),
        );
    }

    #[test]
    fn test_float() {
        let itr = input_type_reference("Query", "field", "floatArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(123.456), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!(123), &[]));
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!("123.4"),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!("123.4"), &[]),
        );
    }

    #[test]
    fn test_id() {
        let itr = input_type_reference("Query", "field", "idArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(1), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!("a"), &[]));
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(123.4),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!(123.4), &[]),
        );
    }

    #[test]
    fn test_boolean() {
        let itr = input_type_reference("Query", "field", "booleanArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(true), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!(false), &[]));
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(1),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!(1), &[]),
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!("true"),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!("true"), &[]),
        );
    }

    #[test]
    fn test_optional() {
        let itr = input_type_reference("Query", "field", "optionalArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(null), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!(123), &[]));
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!("123"),
                input_type_name: itr.as_ref().display_name(),
                path: vec![]
            }]),
            itr.coerce_const_value(&json!("123"), &[]),
        );
    }

    #[test]
    fn test_optional_list() {
        let itr = input_type_reference("Query", "field", "optionalListArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(null), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!(1), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!([1]), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!([1, 2, 3]), &[]));
        assert_eq!(
            Err(vec![
                Error::NoImplicitConversion {
                    value: &json!("b"),
                    input_type_name: "Int".to_string(),
                    path: vec![PathMember::Index(1)]
                },
                Error::NoImplicitConversion {
                    value: &json!(true),
                    input_type_name: "Int".to_string(),
                    path: vec![PathMember::Index(2)]
                },
            ]),
            itr.coerce_const_value(&json!([1, "b", true]), &[]),
        );
    }

    #[test]
    fn test_optional_list_of_list() {
        let itr = input_type_reference("Query", "field", "optionalListOfListArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!(null), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!(1), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!([[1], [2, 3]]), &[]));
        assert_eq!(
            Err(vec![
                Error::NoImplicitConversion {
                    value: &json!(1),
                    input_type_name: "[Int]".to_string(),
                    path: vec![PathMember::Index(0)]
                },
                Error::NoImplicitConversion {
                    value: &json!(2),
                    input_type_name: "[Int]".to_string(),
                    path: vec![PathMember::Index(1)]
                },
                Error::NoImplicitConversion {
                    value: &json!(3),
                    input_type_name: "[Int]".to_string(),
                    path: vec![PathMember::Index(2)]
                },
            ]),
            itr.coerce_const_value(&json!([1, 2, 3]), &[]),
        );
    }

    #[test]
    fn test_enum() {
        let itr = input_type_reference("Query", "field", "enumArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!("FIRST"), &[]));
        assert_eq!(Ok(()), itr.coerce_const_value(&json!("SECOND"), &[]));
        assert_eq!(
            Err(vec![Error::NoEnumMemberWithName {
                name: "first",
                value: &json!("first"),
                enum_type_name: "Choices",
                path: vec![],
            }]),
            itr.coerce_const_value(&json!("first"), &[]),
        );
    }

    #[test]
    fn test_input_object() {
        let itr = input_type_reference("Query", "field", "inputObjectArg");

        assert_eq!(
            Ok(()),
            itr.coerce_const_value(&json!({ "stringArg": "abc" }), &[]),
        );
        assert_eq!(
            Ok(()),
            itr.coerce_const_value(
                &json!({ "stringArg": "abc", "optionalStringArg": "def", "stringArgWithDefault": "ghi" }),
                &[],
            ),
        );
        assert_eq!(
            Err(vec![Error::NoImplicitConversion {
                value: &json!(""),
                input_type_name: itr.as_ref().display_name(),
                path: vec![],
            }]),
            itr.coerce_const_value(&json!(""), &[]),
        );
        assert_eq!(
            Err(vec![Error::NoValueForRequiredFields {
                value: &json!({}),
                field_names: vec!["stringArg"],
                input_object_type_name: "CustomInput",
                path: vec![],
            }]),
            itr.coerce_const_value(&json!({}), &[]),
        );
        assert_eq!(
            Err(vec![Error::NoInputFieldWithName {
                field: &"notDefined".to_owned(),
                input_object_type_name: "CustomInput",
                path: vec![PathMember::Key("notDefined")],
            }]),
            itr.coerce_const_value(&json!({ "stringArg": "abc", "notDefined": "def" }), &[]),
        );
        assert_eq!(
            Err(vec![Error::NullValueForRequiredType {
                value: &json!(null),
                input_type_name: "String!".to_owned(),
                path: vec![PathMember::Key("stringArgWithDefault")],
            }]),
            itr.coerce_const_value(
                &json!({ "stringArg": "abc", "stringArgWithDefault": null }),
                &[]
            ),
        );
    }

    #[test]
    fn test_custom_scalar() {
        let itr = input_type_reference("Query", "field", "decimalArg");

        assert_eq!(Ok(()), itr.coerce_const_value(&json!("123.456"), &[]));
        assert_eq!(
            Err(vec![Error::CustomScalarInvalidValue {
                value: &json!(123.456),
                custom_scalar_type_name: "Decimal",
                message: Cow::Owned("Cannot coerce float to Decimal".to_owned()),
                path: vec![],
            }]),
            itr.coerce_const_value(&json!(123.456), &[]),
        );
    }
}
