use crate::string_value::DisplayStringValue;
use bluejay_core::{
    BooleanValue, EnumValue, FloatValue, IntegerValue, ListValue, ObjectValue, StringValue, Value,
    Variable,
};
use std::fmt::{Error, Write};

pub(crate) trait DisplayValue {
    fn fmt<W: Write>(&self, f: &mut W) -> Result<(), Error>;

    fn to_string(&self) -> String {
        let mut s = String::new();
        self.fmt(&mut s)
            .expect("fmt returned an error unexpectedly");
        s
    }
}

impl<
        const CONST: bool,
        V: Variable,
        I: IntegerValue,
        F: FloatValue,
        S: StringValue,
        B: BooleanValue,
        N,
        E: EnumValue,
        L: ListValue<Self>,
        O: ObjectValue<Self>,
    > DisplayValue for Value<CONST, V, I, F, S, B, N, E, L, O>
{
    fn fmt<W: Write>(&self, f: &mut W) -> Result<(), Error> {
        match self {
            Self::Boolean(b) => write!(f, "{}", b.to_bool()),
            Self::Enum(e) => write!(f, "{}", e.as_ref()),
            Self::Float(fl) => {
                let fl = fl.to_f64();
                if fl.fract().abs() < 1e-10 {
                    write!(f, "{fl:.1}")
                } else {
                    write!(f, "{fl}")
                }
            }
            Self::Integer(i) => write!(f, "{}", i.to_i32()),
            Self::List(l) => {
                write!(f, "[")?;
                l.as_ref().iter().enumerate().try_for_each(|(idx, el)| {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    el.fmt(f)
                })?;
                write!(f, "]")
            }
            Self::Null(_) => write!(f, "null"),
            Self::Object(o) => {
                write!(f, "{{ ")?;

                o.iter().enumerate().try_for_each(|(idx, (key, value))| {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: ")?;
                    value.fmt(f)
                })?;

                write!(f, " }}")
            }
            Self::String(s) => DisplayStringValue::fmt(s.as_ref(), f),
            Self::Variable(v) => write!(f, "${}", v.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayValue;
    use bluejay_parser::ast::{Parse, VariableValue};

    macro_rules! assert_prints {
        ($val:literal) => {
            let parsed = VariableValue::parse($val).unwrap();
            assert_eq!($val, DisplayValue::to_string(&parsed));
        };
        ($out:literal, $in:literal) => {
            let parsed = VariableValue::parse($in).unwrap();
            assert_eq!($out, DisplayValue::to_string(&parsed));
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
        assert_prints!(r#""\"\\\/\b\n\f\r\t""#);
        assert_prints!(r#""🔥""#);
    }

    #[test]
    fn test_variable() {
        assert_prints!("$foo");
    }
}
