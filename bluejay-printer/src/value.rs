use crate::string_value::StringValuePrinter;
use bluejay_core::{AsIter, ObjectValue, Value, ValueReference, Variable};
use std::fmt::{Display, Formatter, Result};

pub struct ValuePrinter<'a, const CONST: bool, V: Value<CONST>>(&'a V);

impl<'a, const CONST: bool, V: Value<CONST>> ValuePrinter<'a, CONST, V> {
    pub fn new(value: &'a V) -> Self {
        Self(value)
    }

    pub fn to_string(value: &'a V) -> String {
        Self::new(value).to_string()
    }
}

impl<'a, const CONST: bool, V: Value<CONST>> Display for ValuePrinter<'a, CONST, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(value) = *self;
        match value.as_ref() {
            ValueReference::Boolean(b) => write!(f, "{}", b),
            ValueReference::Enum(e) => write!(f, "{}", e),
            ValueReference::Float(fl) => {
                if fl.fract().abs() < 1e-10 {
                    write!(f, "{fl:.1}")
                } else {
                    write!(f, "{fl}")
                }
            }
            ValueReference::Integer(i) => write!(f, "{}", i),
            ValueReference::List(l) => {
                write!(f, "[")?;
                l.iter().enumerate().try_for_each(|(idx, el)| {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", Self::new(el))
                })?;
                write!(f, "]")
            }
            ValueReference::Null => write!(f, "null"),
            ValueReference::Object(o) => {
                write!(f, "{{ ")?;

                o.iter().enumerate().try_for_each(|(idx, (key, value))| {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key.as_ref(), Self::new(value))
                })?;

                write!(f, " }}")
            }
            ValueReference::String(s) => write!(f, "{}", StringValuePrinter::new(s)),
            ValueReference::Variable(v) => write!(f, "${}", v.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValuePrinter;
    use bluejay_parser::ast::{Parse, VariableValue};

    macro_rules! assert_prints {
        ($val:literal) => {
            let parsed = VariableValue::parse($val).unwrap();
            assert_eq!($val, ValuePrinter::new(&parsed).to_string());
        };
        ($out:literal, $in:literal) => {
            let parsed = VariableValue::parse($in).unwrap();
            assert_eq!($out, ValuePrinter::new(&parsed).to_string());
        };
    }

    #[test]
    fn test_bool() {
        assert_prints!("true");
        assert_prints!("false");
    }

    #[test]
    fn test_enum() {
        assert_prints!("ONE");
    }

    #[test]
    fn test_float() {
        assert_prints!("1.0");
        assert_prints!("3.14159");
        assert_prints!("-1.23456");
        assert_prints!("10000.0", "1e4");
        assert_prints!("0.0");
    }

    #[test]
    fn test_int() {
        assert_prints!("1");
        assert_prints!("0");
        assert_prints!("-100");
    }

    #[test]
    fn test_list() {
        assert_prints!("[1, 2, 3]");
        assert_prints!("[]");
        assert_prints!("[[]]");
    }

    #[test]
    fn test_null() {
        assert_prints!("null");
    }

    #[test]
    fn test_object() {
        assert_prints!("{ foo: 1, bar: 2 }");
    }

    #[test]
    fn test_string() {
        assert_prints!(r#""""#);
        assert_prints!(r#""\"\\/\b\n\f\r\t""#, r#""\"\\\/\b\n\f\r\t""#);
        assert_prints!(r#""🔥""#);
    }

    #[test]
    fn test_variable() {
        assert_prints!("$foo");
    }
}
