use std::fmt;

/// A lossless integer value, independent of the range of the GraphQL `Int` scalar.
///
/// Parsed literals borrow their decimal representation, including the sign of
/// `-0`. Native values support adapters such as `serde_json` without allocating a
/// decimal string. Equality is numeric, so `-0`, `0`, and native zero are equal.
#[derive(Clone, Copy, Debug)]
pub struct IntegerValue<'a>(Representation<'a>);

#[derive(Clone, Copy, Debug)]
enum Representation<'a> {
    Literal(&'a str),
    Signed(i64),
    Unsigned(u64),
}

impl<'a> IntegerValue<'a> {
    /// Borrow a GraphQL integer literal whose syntax has already been validated.
    ///
    /// This does not scan or validate the text. The caller must ensure it matches
    /// `-?(0|[1-9][0-9]*)` in full. Use `TryFrom<&str>` for unvalidated input.
    #[inline]
    pub const fn from_validated_literal(value: &'a str) -> Self {
        Self(Representation::Literal(value))
    }

    /// Whether the numeric value is below zero. Parsed `-0` is not negative.
    pub fn is_negative(self) -> bool {
        match self.0 {
            Representation::Literal(value) => value.starts_with('-') && value != "-0",
            Representation::Signed(value) => value < 0,
            Representation::Unsigned(_) => false,
        }
    }

    /// Return the value only if it fits in the GraphQL `Int` scalar's range.
    pub fn as_i32(self) -> Option<i32> {
        self.as_i64().and_then(|value| value.try_into().ok())
    }

    /// Return the value only if it fits in a signed 64-bit integer.
    pub fn as_i64(self) -> Option<i64> {
        match self.0 {
            Representation::Literal(value) => value.parse().ok(),
            Representation::Signed(value) => Some(value),
            Representation::Unsigned(value) => value.try_into().ok(),
        }
    }

    pub fn as_u64(self) -> Option<u64> {
        match self.0 {
            Representation::Literal("-0") => Some(0),
            Representation::Literal(value) => value.parse().ok(),
            Representation::Signed(value) => value.try_into().ok(),
            Representation::Unsigned(value) => Some(value),
        }
    }

    /// Convert to a finite float, rounding as necessary. Return `None` on overflow.
    pub fn as_f64(self) -> Option<f64> {
        let value = match self.0 {
            Representation::Literal(value) => value.parse::<f64>().ok()?,
            Representation::Signed(value) => value as f64,
            Representation::Unsigned(value) => value as f64,
        };
        value.is_finite().then_some(value)
    }
}

/// The source text does not match the GraphQL integer-literal grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidIntegerValue;

impl fmt::Display for InvalidIntegerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid GraphQL integer literal")
    }
}

impl std::error::Error for InvalidIntegerValue {}

impl<'a> TryFrom<&'a str> for IntegerValue<'a> {
    type Error = InvalidIntegerValue;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let digits = value.strip_prefix('-').unwrap_or(value).as_bytes();
        if digits == b"0"
            || (matches!(digits.first(), Some(b'1'..=b'9'))
                && digits[1..].iter().all(u8::is_ascii_digit))
        {
            Ok(Self::from_validated_literal(value))
        } else {
            Err(InvalidIntegerValue)
        }
    }
}

impl From<i32> for IntegerValue<'_> {
    fn from(value: i32) -> Self {
        Self(Representation::Signed(value.into()))
    }
}

impl From<i64> for IntegerValue<'_> {
    fn from(value: i64) -> Self {
        Self(Representation::Signed(value))
    }
}

impl From<u64> for IntegerValue<'_> {
    fn from(value: u64) -> Self {
        Self(Representation::Unsigned(value))
    }
}

impl fmt::Display for IntegerValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Representation::Literal(value) => f.write_str(value),
            Representation::Signed(value) => value.fmt(f),
            Representation::Unsigned(value) => value.fmt(f),
        }
    }
}

impl PartialEq for IntegerValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(lhs), Some(rhs)) = (self.as_i64(), other.as_i64()) {
            lhs == rhs
        } else if let (Some(lhs), Some(rhs)) = (self.as_u64(), other.as_u64()) {
            lhs == rhs
        } else {
            // Outside native ranges, both values must be valid decimal literals.
            // Their representation is unique (the -0 case was handled above).
            matches!((self.0, other.0), (Representation::Literal(lhs), Representation::Literal(rhs)) if lhs == rhs)
        }
    }
}

impl Eq for IntegerValue<'_> {}

#[cfg(test)]
mod tests {
    use super::{IntegerValue, InvalidIntegerValue};

    #[test]
    fn literals_are_lossless_and_conversions_are_checked() {
        for (literal, i32_value, i64_value, u64_value) in [
            ("0", Some(0), Some(0), Some(0)),
            ("-0", Some(0), Some(0), Some(0)),
            (
                "2147483647",
                Some(i32::MAX),
                Some(2147483647),
                Some(2147483647),
            ),
            ("-2147483648", Some(i32::MIN), Some(-2147483648), None),
            ("2147483648", None, Some(2147483648), Some(2147483648)),
            ("-2147483649", None, Some(-2147483649), None),
            (
                "9223372036854775807",
                None,
                Some(i64::MAX),
                Some(i64::MAX as u64),
            ),
            ("-9223372036854775808", None, Some(i64::MIN), None),
            ("9223372036854775808", None, None, Some(1 << 63)),
            ("18446744073709551615", None, None, Some(u64::MAX)),
            ("18446744073709551616", None, None, None),
            ("-9223372036854775809", None, None, None),
        ] {
            for value in [
                IntegerValue::try_from(literal).unwrap(),
                IntegerValue::from_validated_literal(literal),
            ] {
                assert_eq!(literal, value.to_string());
                assert_eq!(i32_value, value.as_i32(), "{literal}");
                assert_eq!(i64_value, value.as_i64(), "{literal}");
                assert_eq!(u64_value, value.as_u64(), "{literal}");
                assert!(value.as_f64().unwrap().is_finite());
            }
        }
        for literal in ["9".repeat(1000), format!("-{}", "9".repeat(1000))] {
            for value in [
                IntegerValue::try_from(literal.as_str()).unwrap(),
                IntegerValue::from_validated_literal(&literal),
            ] {
                assert_eq!(literal, value.to_string());
                assert_eq!(None, value.as_i64());
                assert_eq!(None, value.as_u64());
                assert_eq!(None, value.as_f64());
            }
        }
    }

    #[test]
    fn sign_is_numeric_across_representations() {
        for (literal, negative) in [
            ("0", false),
            ("-0", false),
            ("1", false),
            ("-1", true),
            ("18446744073709551616", false),
            ("-18446744073709551616", true),
        ] {
            assert_eq!(
                negative,
                IntegerValue::try_from(literal).unwrap().is_negative(),
                "{literal}"
            );
        }
        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            assert_eq!(value < 0, IntegerValue::from(value).is_negative());
        }
        for value in [0, 1, u64::MAX] {
            assert!(!IntegerValue::from(value).is_negative());
        }
    }

    #[test]
    fn invalid_literals_are_rejected() {
        for literal in [
            "", "-", "+1", "00", "01", "-01", "--1", " 1", "1 ", "1.0", "1e3", "1a", "1١",
        ] {
            assert_eq!(
                Err(InvalidIntegerValue),
                IntegerValue::try_from(literal),
                "{literal:?}"
            );
        }
    }

    #[test]
    fn equality_is_numeric_across_representations() {
        for value in [i64::MIN, i64::MAX, -1, 0, 1, 2147483648] {
            let literal = value.to_string();
            let parsed = IntegerValue::try_from(literal.as_str()).unwrap();
            let native = IntegerValue::from(value);
            assert_eq!(parsed, native);
            assert_eq!(native, parsed);
            assert_eq!(literal, native.to_string());
        }
        for value in [i64::MAX as u64, u64::MAX] {
            let literal = value.to_string();
            let parsed = IntegerValue::try_from(literal.as_str()).unwrap();
            let native = IntegerValue::from(value);
            assert_eq!(parsed, native);
            assert_eq!(native, parsed);
            assert_eq!(literal, native.to_string());
        }
        let zero = IntegerValue::try_from("-0").unwrap();
        assert_eq!(zero, IntegerValue::try_from("0").unwrap());
        assert_eq!(zero, IntegerValue::from(0i32));
        assert_eq!(zero, IntegerValue::from(0u64));
        assert_ne!(IntegerValue::from(-1i64), IntegerValue::from(u64::MAX));
        let huge = "9".repeat(1000);
        let other = format!("{huge}0");
        assert_eq!(
            IntegerValue::try_from(huge.as_str()),
            IntegerValue::try_from(huge.as_str())
        );
        assert_ne!(
            IntegerValue::try_from(huge.as_str()),
            IntegerValue::try_from(other.as_str())
        );
    }
}
