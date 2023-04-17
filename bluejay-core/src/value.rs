use crate::AsIter;
use std::collections::HashMap;

pub trait ObjectValue<const CONST: bool> {
    type Key: AsRef<str>;
    type Value: AbstractValue<CONST, Object = Self>;
    type Iterator<'a>: Iterator<Item = (&'a Self::Key, &'a Self::Value)>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_>;
}

pub trait ListValue<const CONST: bool>: AsIter<Item = Self::Value> {
    type Value: AbstractValue<CONST, List = Self>;
}

pub trait AbstractValue<const CONST: bool> {
    type List: ListValue<CONST, Value = Self>;
    type Object: ObjectValue<CONST, Value = Self>;

    fn as_ref(&self) -> ValueFromAbstract<'_, CONST, Self>;
}

pub trait AbstractConstValue: AbstractValue<true> {}
pub trait AbstractVariableValue: AbstractValue<false> {}

impl<T: AbstractValue<true>> AbstractConstValue for T {}
impl<T: AbstractValue<false>> AbstractVariableValue for T {}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Value<'a, const CONST: bool, L: ListValue<CONST>, O: ObjectValue<CONST>> {
    Variable(&'a str),
    Integer(i32),
    Float(f64),
    String(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(&'a L),
    Object(&'a O),
}

impl<'a, const CONST: bool, L: ListValue<CONST>, O: ObjectValue<CONST>> Clone
    for Value<'a, CONST, L, O>
{
    fn clone(&self) -> Self {
        match self {
            Self::Variable(v) => Self::Variable(v),
            Self::Integer(i) => Self::Integer(*i),
            Self::Float(f) => Self::Float(*f),
            Self::String(s) => Self::String(s),
            Self::Boolean(b) => Self::Boolean(*b),
            Self::Null => Self::Null,
            Self::Enum(e) => Self::Enum(e),
            Self::List(l) => Self::List(l),
            Self::Object(o) => Self::Object(o),
        }
    }
}

impl<'a, const CONST: bool, L: ListValue<CONST>, O: ObjectValue<CONST>> Copy
    for Value<'a, CONST, L, O>
{
}

pub type ValueFromAbstract<'a, const CONST: bool, T> =
    Value<'a, CONST, <T as AbstractValue<CONST>>::List, <T as AbstractValue<CONST>>::Object>;

impl<'a, const CONST: bool, L: ListValue<CONST>, O: ObjectValue<CONST, Value = L::Value>>
    std::cmp::PartialEq for Value<'a, CONST, L, O>
{
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Variable(v) => {
                matches!(other, Self::Variable(other_v) if v == other_v)
            }
            Self::Integer(i) => {
                matches!(other, Self::Integer(other_i) if i == other_i)
            }
            Self::Float(f) => {
                matches!(other, Self::Float(other_f) if f == other_f)
            }
            Self::String(s) => {
                matches!(other, Self::String(other_s) if s == other_s)
            }
            Self::Boolean(b) => {
                matches!(other, Self::Boolean(other_b) if b == other_b)
            }
            Self::Null => matches!(other, Self::Null),
            Self::Enum(e) => matches!(other, Self::Enum(other_e) if e == other_e),
            Self::List(l) => {
                matches!(other, Self::List(other_l) if Vec::from_iter(l.iter().map(AbstractValue::as_ref)) == Vec::from_iter(other_l.iter().map(AbstractValue::as_ref)))
            }
            Self::Object(o) => matches!(other, Self::Object(other_o) if {
                let lhs: HashMap<&str, _> = HashMap::from_iter(o.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
                let rhs: HashMap<&str, _> = HashMap::from_iter(other_o.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
                lhs == rhs
            }),
        }
    }
}
