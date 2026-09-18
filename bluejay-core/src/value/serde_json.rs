use crate::{ObjectValue, Value, ValueReference, Variable};
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

impl<const CONST: bool> Value<CONST> for JsonValue {
    type List = Vec<JsonValue>;
    type Object = Map<String, JsonValue>;
    type Variable = Never;

    fn as_ref(&self) -> ValueReference<'_, CONST, Self> {
        match self {
            Self::Null => ValueReference::Null,
            Self::Bool(b) => ValueReference::Boolean(*b),
            Self::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ValueReference::Integer(i.into())
                } else if let Some(i) = n.as_u64() {
                    ValueReference::Integer(i.into())
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

#[cfg(test)]
mod tests {
    use super::{JsonValue, Value, ValueReference};
    use serde_json::json;

    #[test]
    fn native_json_integers_keep_their_kind_and_precision() {
        for value in [
            json!(0),
            json!(2147483648i64),
            json!(-2147483649i64),
            json!(9007199254740993i64),
            json!(i64::MIN),
            json!(i64::MAX),
            json!(u64::MAX),
        ] {
            let ValueReference::Integer(integer) = <JsonValue as Value<true>>::as_ref(&value)
            else {
                panic!("expected an integer for {value}");
            };
            assert_eq!(value.to_string(), integer.to_string());
        }
    }

    #[test]
    fn json_floats_do_not_become_integers() {
        for value in [json!(1.0), json!(-0.0), json!(2147483648.0)] {
            assert!(matches!(
                <JsonValue as Value<true>>::as_ref(&value),
                ValueReference::Float(_)
            ));
        }
    }
}
