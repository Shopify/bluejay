use crate::Context;
use bluejay_core::{
    executable::ExecutableDocument, AsIter, ObjectValue, Value, ValueReference, Variable,
};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum MergedValue<'a, const CONST: bool> {
    Variable(Cow<'a, str>),
    Integer(i32),
    Float(f64),
    String(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(Vec<Self>),
    Object(Vec<(&'a str, Self)>),
}

impl<'a, const CONST: bool> Value<CONST> for MergedValue<'a, CONST> {
    type Variable = Cow<'a, str>;
    type List = Vec<Self>;
    type Object = Vec<(&'a str, Self)>;

    fn as_ref(&self) -> ValueReference<'_, CONST, Self> {
        match self {
            Self::Variable(v) => ValueReference::Variable(v),
            Self::Integer(i) => ValueReference::Integer(*i),
            Self::Float(f) => ValueReference::Float(*f),
            Self::String(s) => ValueReference::String(s),
            Self::Boolean(b) => ValueReference::Boolean(*b),
            Self::Null => ValueReference::Null,
            Self::Enum(e) => ValueReference::Enum(e),
            Self::List(l) => ValueReference::List(l),
            Self::Object(o) => ValueReference::Object(o),
        }
    }
}

impl<'a, const CONST: bool> MergedValue<'a, CONST> {
    pub(crate) fn new<E: ExecutableDocument>(
        value: &'a impl Value<CONST>,
        context: &Context<'a, E>,
    ) -> Self {
        match value.as_ref() {
            ValueReference::Variable(v) => {
                context.variable_replacement(v.name()).map_or_else(
                    || Self::Variable(context.variable_name(v.name())),
                    Self::String,
                )
            },
            ValueReference::Integer(i) => Self::Integer(i),
            ValueReference::Float(f) => Self::Float(f),
            ValueReference::String(s) => Self::String(s),
            ValueReference::Boolean(b) => Self::Boolean(b),
            ValueReference::Null => Self::Null,
            ValueReference::Enum(e) => Self::Enum(e),
            ValueReference::List(l) => {
                Self::List(l.iter().map(|v| Self::new(v, context)).collect())
            }
            ValueReference::Object(o) => Self::Object(
                o.iter()
                    .map(|(k, v)| (k.as_ref(), Self::new(v, context)))
                    .collect(),
            ),
        }
    }
}
