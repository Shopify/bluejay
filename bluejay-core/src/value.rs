use crate::AsIter;
use enum_as_inner::EnumAsInner;
use std::collections::HashMap;

#[cfg(feature = "serde_json")]
mod serde_json;

pub trait ObjectValue<const CONST: bool>: std::fmt::Debug {
    type Key: AsRef<str> + PartialEq + std::fmt::Debug;
    type Value: Value<CONST, Object = Self>;
    type Iterator<'a>: Iterator<Item = (&'a Self::Key, &'a Self::Value)>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_>;
}

impl<
        const CONST: bool,
        K: AsRef<str> + PartialEq + std::fmt::Debug,
        V: Value<CONST, Object = Vec<(K, V)>> + std::fmt::Debug,
    > ObjectValue<CONST> for Vec<(K, V)>
{
    type Key = K;
    type Value = V;
    type Iterator<'a> =
        std::iter::Map<std::slice::Iter<'a, (K, V)>, fn(&'a (K, V)) -> (&'a K, &'a V)> where Self: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        self.as_slice().iter().map(|(k, v)| (k, v))
    }
}

pub trait ListValue<const CONST: bool>: AsIter<Item = Self::Value> + std::fmt::Debug {
    type Value: Value<CONST, List = Self>;
}

impl<const CONST: bool, T: Value<CONST, List = Vec<T>> + std::fmt::Debug> ListValue<CONST>
    for Vec<T>
{
    type Value = T;
}

pub trait Variable {
    fn name(&self) -> &str;
}

impl<T: AsRef<str>> Variable for T {
    fn name(&self) -> &str {
        self.as_ref()
    }
}

pub trait Value<const CONST: bool>: Sized {
    type List: ListValue<CONST, Value = Self>;
    type Object: ObjectValue<CONST, Value = Self>;
    type Variable: Variable;

    fn as_ref(&self) -> ValueReference<'_, CONST, Self>;

    fn can_coerce_string_value_to_enum() -> bool {
        false
    }
}

pub trait ConstValue: Value<true> {}
pub trait VariableValue: Value<false> {}

impl<T: Value<true>> ConstValue for T {}
impl<T: Value<false>> VariableValue for T {}

#[derive(Debug, strum::IntoStaticStr, EnumAsInner)]
#[strum(serialize_all = "lowercase")]
pub enum ValueReference<'a, const CONST: bool, V: Value<CONST>> {
    Variable(&'a V::Variable),
    Integer(i32),
    Float(f64),
    String(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(&'a V::List),
    Object(&'a V::Object),
}

impl<'a, const CONST: bool, V: Value<CONST>> ValueReference<'a, CONST, V> {
    pub fn variant(&self) -> &'static str {
        self.into()
    }
}

impl<'a, const CONST: bool, V: Value<CONST>> Clone for ValueReference<'a, CONST, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, const CONST: bool, V: Value<CONST>> Copy for ValueReference<'a, CONST, V> {}

impl<'a, const CONST: bool, V: Value<CONST>> std::cmp::PartialEq for ValueReference<'a, CONST, V> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Variable(v) => {
                matches!(other, Self::Variable(other_v) if v.name() == other_v.name())
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
                matches!(other, Self::List(other_l) if itertools::equal(l.iter().map(Value::as_ref), other_l.iter().map(Value::as_ref)))
            }
            Self::Object(o) => matches!(other, Self::Object(other_o) if {
                let lhs: HashMap<&str, _> = HashMap::from_iter(o.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
                let rhs: HashMap<&str, _> = HashMap::from_iter(other_o.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
                lhs == rhs
            }),
        }
    }
}
