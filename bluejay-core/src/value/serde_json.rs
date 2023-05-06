use crate::{AbstractValue, ListValue, ObjectValue, Value, ValueFromAbstract, Variable};
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

impl<const CONST: bool> AbstractValue<CONST> for JsonValue {
    type List = Vec<JsonValue>;
    type Object = Map<String, JsonValue>;
    type Variable = Never;

    fn as_ref(&self) -> ValueFromAbstract<'_, CONST, Self> {
        match self {
            Self::Null => Value::Null,
            Self::Bool(b) => Value::Boolean(*b),
            Self::Number(n) => {
                if let Some(i) = n.as_i64().and_then(|i| i32::try_from(i).ok()) {
                    Value::Integer(i)
                } else {
                    Value::Float(n.as_f64().expect("Json numeric values must be finite"))
                }
            }
            Self::String(s) => Value::String(s),
            Self::Array(a) => Value::List(a),
            Self::Object(o) => Value::Object(o),
        }
    }

    fn can_coerce_string_value_to_enum() -> bool {
        true
    }
}
