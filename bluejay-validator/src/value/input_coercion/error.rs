use crate::value::input_coercion::PathMember;
use bluejay_core::{ObjectValue, Value};
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::Value as ParserValue,
    error::{Annotation, Error as ParserError},
    HasSpan,
};
#[cfg(feature = "parser-integration")]
use itertools::Itertools;
use std::borrow::Cow;

#[derive(PartialEq, Debug)]
pub enum Error<'a, const CONST: bool, V: Value<CONST>> {
    NullValueForRequiredType {
        value: &'a V,
        input_type_name: String,
        path: Vec<PathMember<'a>>,
    },
    NoImplicitConversion {
        value: &'a V,
        input_type_name: String,
        path: Vec<PathMember<'a>>,
    },
    NoEnumMemberWithName {
        name: &'a str,
        value: &'a V,
        enum_type_name: &'a str,
        path: Vec<PathMember<'a>>,
    },
    NoValueForRequiredFields {
        value: &'a V,
        field_names: Vec<&'a str>,
        input_object_type_name: &'a str,
        path: Vec<PathMember<'a>>,
    },
    NonUniqueFieldNames {
        value: &'a V,
        field_name: &'a str,
        keys: Vec<&'a <V::Object as ObjectValue<CONST>>::Key>,
        path: Vec<PathMember<'a>>,
    },
    NoInputFieldWithName {
        field: &'a <V::Object as ObjectValue<CONST>>::Key,
        input_object_type_name: &'a str,
        path: Vec<PathMember<'a>>,
    },
    CustomScalarInvalidValue {
        value: &'a V,
        custom_scalar_type_name: &'a str,
        message: Cow<'static, str>,
        path: Vec<PathMember<'a>>,
    },
    #[cfg(feature = "one-of-input-objects")]
    OneOfInputNullValues {
        value: &'a V,
        input_object_type_name: &'a str,
        null_entries: Vec<(&'a <V::Object as ObjectValue<CONST>>::Key, &'a V)>,
        path: Vec<PathMember<'a>>,
    },
    #[cfg(feature = "one-of-input-objects")]
    OneOfInputNotSingleNonNullValue {
        value: &'a V,
        input_object_type_name: &'a str,
        non_null_entries: Vec<(&'a <V::Object as ObjectValue<CONST>>::Key, &'a V)>,
        path: Vec<PathMember<'a>>,
    },
}

#[cfg(feature = "parser-integration")]
impl<'a, const CONST: bool> From<Error<'a, CONST, ParserValue<'a, CONST>>> for ParserError {
    fn from(value: Error<'a, CONST, ParserValue<'a, CONST>>) -> Self {
        match value {
            Error::NullValueForRequiredType {
                value,
                input_type_name,
                ..
            } => Self::new(
                format!("Got null when non-null value of type {input_type_name} was expected"),
                Some(Annotation::new(
                    "Expected non-null value",
                    value.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoImplicitConversion {
                value,
                input_type_name,
                ..
            } => Self::new(
                format!("No implicit conversion of {value} to {input_type_name}"),
                Some(Annotation::new(
                    format!("No implicit conversion to {input_type_name}"),
                    value.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoEnumMemberWithName {
                name,
                value,
                enum_type_name,
                ..
            } => Self::new(
                format!("No member `{name}` on enum {enum_type_name}"),
                Some(Annotation::new(
                    format!("No such member on enum {enum_type_name}"),
                    value.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoValueForRequiredFields {
                value,
                field_names,
                input_object_type_name,
                ..
            } => {
                let joined_field_names = field_names.into_iter().join(", ");
                Self::new(
                    format!("No value for required fields on input type {input_object_type_name}: {joined_field_names}"),
                    Some(Annotation::new(
                        format!("No value for required fields: {joined_field_names}"),
                        value.span().clone(),
                    )),
                    Vec::new(),
                )
            }
            Error::NonUniqueFieldNames {
                field_name, keys, ..
            } => Self::new(
                format!("Object with multiple entries for field {field_name}"),
                None,
                Vec::from_iter(
                    keys.into_iter()
                        .map(|key| Annotation::new("Entry for field", key.span().clone())),
                ),
            ),
            Error::NoInputFieldWithName {
                field,
                input_object_type_name,
                ..
            } => Self::new(
                format!(
                    "No field with name {} on input type {input_object_type_name}",
                    field.as_ref()
                ),
                Some(Annotation::new(
                    format!("No field with this name on input type {input_object_type_name}"),
                    field.span().clone(),
                )),
                Vec::new(),
            ),
            Error::CustomScalarInvalidValue { value, message, .. } => Self::new(
                message.clone(),
                Some(Annotation::new(message, value.span().clone())),
                Vec::new(),
            ),
            #[cfg(feature = "one-of-input-objects")]
            Error::OneOfInputNullValues { value, input_object_type_name, null_entries, .. } => Self::new(
                format!("Multiple entries with null values for oneOf input object {input_object_type_name}"),
                Some(Annotation::new(
                    "oneOf input object must not contain any null values",
                    value.span().clone(),
                )),
                null_entries.into_iter().map(|(key, value)| {
                    Annotation::new(
                        "Entry with null value",
                        key.span().merge(value.span()),
                    )
                }).collect(),
            ),
            #[cfg(feature = "one-of-input-objects")]
            Error::OneOfInputNotSingleNonNullValue { value, input_object_type_name, non_null_entries, .. } => Self::new(
                format!("Got {} entries with non-null values for oneOf input object {input_object_type_name}", non_null_entries.len()),
                Some(Annotation::new(
                    "oneOf input object must contain single non-null",
                    value.span().clone(),
                )),
                non_null_entries.into_iter().map(|(key, value)| {
                    Annotation::new(
                        "Entry with non-null value",
                        key.span().merge(value.span()),
                    )
                }).collect(),
            ),
        }
    }
}
