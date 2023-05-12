use crate::{ListValue, ObjectValue, Value, ValueReference, Variable};
use serde_json::{map, Map, Value as JsonValue};

pub enum Never {}

impl Variable for Never {
    fn name(&self) -> &str {
        unreachable!()
    }
}

impl<const CONST: bool> ObjectValue<CONST> for Map<String, JsonValue> {
    type Key = String;
    type Value = JsonValue;
    type Iterator<'a> = map::Iter<'a>;

    fn iter(&self) -> Self::Iterator<'_> {
        self.iter()
    }
}

impl<const CONST: bool> ListValue<CONST> for Vec<JsonValue> {
    type Value = JsonValue;
}

impl<const CONST: bool> Value<CONST> for JsonValue {
    type List = Vec<JsonValue>;
    type Object = Map<String, JsonValue>;
    type Variable = Never;

    fn as_ref(&self) -> ValueReference<'_, CONST, Self> {
        match self {
            Self::Null => ValueReference::Null,
            Self::Bool(b) => ValueReference::Boolean(*b),
            Self::Number(n) => {
                if let Some(i) = n.as_i64().and_then(|i| i32::try_from(i).ok()) {
                    ValueReference::Integer(i)
                } else {
                    ValueReference::Float(n.as_f64().expect("Json numeric values must be finite"))
                }
            }
            Self::String(s) => ValueReference::String(s),
            Self::Array(a) => ValueReference::List(a),
            Self::Object(o) => ValueReference::Object(o),
        }
    }

    fn can_coerce_string_value_to_enum() -> bool {
        true
    }
}
