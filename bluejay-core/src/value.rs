use crate::AsIter;
use enum_as_inner::EnumAsInner;
use fnv::FnvHashMap;

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

pub trait ListValue<const CONST: bool>: AsIter<Item = Self::Value> + std::fmt::Debug {
    type Value: Value<CONST, List = Self>;
}

pub trait Variable {
    fn name(&self) -> &str;
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
                let lhs: FnvHashMap<&str, _> = FnvHashMap::from_iter(o.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
                let rhs: FnvHashMap<&str, _> = FnvHashMap::from_iter(other_o.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
                lhs == rhs
            }),
        }
    }
}
