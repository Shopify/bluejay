use crate::ast::{FromTokens, ParseError, Tokens, TryFromTokens, Variable};
use crate::lexical_token::{FloatValue, HasSpan, IntValue, Name, PunctuatorType, StringValue};
use crate::Span;
use bluejay_core::AsIter;

pub type Value<'a, const CONST: bool> = bluejay_core::Value<
    CONST,
    Variable<'a>,
    IntValue,
    FloatValue,
    StringValue,
    BooleanValue,
    Name<'a>,
    Name<'a>,
    ListValue<'a, CONST>,
    ObjectValue<'a, CONST>,
>;

impl<'a, const CONST: bool> FromTokens<'a> for Value<'a, CONST> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        None.or_else(|| {
            if CONST {
                None
            } else {
                Variable::try_from_tokens(tokens).map(|res| res.map(Self::Variable))
            }
        })
        .or_else(|| {
            tokens
                .next_if_int_value()
                .map(|i| Ok(Self::Integer(i)))
                .or_else(|| tokens.next_if_float_value().map(|f| Ok(Self::Float(f))))
                .or_else(|| tokens.next_if_string_value().map(|s| Ok(Self::String(s))))
                .or_else(|| {
                    tokens.next_if_name().map(|name| {
                        Ok(match name.as_str() {
                            "true" => Self::Boolean(BooleanValue {
                                value: true,
                                span: name.span(),
                            }),
                            "false" => Self::Boolean(BooleanValue {
                                value: false,
                                span: name.span(),
                            }),
                            "null" => Self::Null(name),
                            _ => Self::Enum(name),
                        })
                    })
                })
                .or_else(|| {
                    tokens
                        .next_if_punctuator(PunctuatorType::OpenSquareBracket)
                        .map(|open_span| {
                            let mut list: Vec<Self> = Vec::new();
                            let close_span = loop {
                                if let Some(close_span) =
                                    tokens.next_if_punctuator(PunctuatorType::CloseSquareBracket)
                                {
                                    break close_span;
                                }
                                list.push(Self::from_tokens(tokens)?);
                            };
                            let span = open_span.merge(&close_span);
                            Ok(Self::List(ListValue {
                                elements: list,
                                span,
                            }))
                        })
                })
                .or_else(|| {
                    tokens
                        .next_if_punctuator(PunctuatorType::OpenBrace)
                        .map(|open_span| {
                            let mut object: Vec<_> = Vec::new();
                            let close_span = loop {
                                if let Some(close_span) =
                                    tokens.next_if_punctuator(PunctuatorType::CloseBrace)
                                {
                                    break close_span;
                                }
                                let name = tokens.expect_name()?;
                                tokens.expect_punctuator(PunctuatorType::Colon)?;
                                let value = Self::from_tokens(tokens)?;
                                object.push((name, value));
                            };
                            let span = open_span.merge(&close_span);
                            Ok(Self::Object(ObjectValue {
                                fields: object,
                                span,
                            }))
                        })
                })
        })
        .unwrap_or_else(|| {
            Err(tokens
                .next()
                .map(|token| ParseError::UnexpectedToken { span: token.span() })
                .unwrap_or_else(|| tokens.unexpected_eof()))
        })
    }
}

impl<'a, const CONST: bool> HasSpan for Value<'a, CONST> {
    fn span(&self) -> Span {
        match self {
            Self::Boolean(b) => b.span.clone(),
            Self::Enum(e) => e.span(),
            Self::Float(f) => f.span(),
            Self::Integer(i) => i.span(),
            Self::List(l) => l.span.clone(),
            Self::Null(n) => n.span(),
            Self::Object(o) => o.span.clone(),
            Self::String(s) => s.span(),
            Self::Variable(v) => v.span(),
        }
    }
}

pub type ConstValue<'a> = Value<'a, true>;
pub type VariableValue<'a> = Value<'a, false>;

#[derive(Debug)]
pub struct BooleanValue {
    value: bool,
    span: Span,
}

impl bluejay_core::BooleanValue for BooleanValue {
    fn to_bool(&self) -> bool {
        self.value
    }
}

#[derive(Debug)]
pub struct ListValue<'a, const CONST: bool> {
    elements: Vec<Value<'a, CONST>>,
    span: Span,
}

impl<'a, const CONST: bool> AsRef<[Value<'a, CONST>]> for ListValue<'a, CONST> {
    fn as_ref(&self) -> &[Value<'a, CONST>] {
        &self.elements
    }
}

impl<'a, const CONST: bool> bluejay_core::ListValue<Value<'a, CONST>> for ListValue<'a, CONST> {}

#[derive(Debug)]
pub struct ObjectValue<'a, const CONST: bool> {
    fields: Vec<(Name<'a>, Value<'a, CONST>)>,
    span: Span,
}

impl<'a, const CONST: bool> bluejay_core::ObjectValue<Value<'a, CONST>> for ObjectValue<'a, CONST> {
    type Iterator<'b> = std::iter::Map<std::slice::Iter<'b, (Name<'a>, Value<'a, CONST>)>, fn(&'b (Name<'a>, Value<'a, CONST>)) -> (&'b str, &'b Value<'a, CONST>)> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields.iter().map(|(key, value)| (key.as_str(), value))
    }
}

impl<'a, const CONST: bool> AsIter for ObjectValue<'a, CONST> {
    type Item = (Name<'a>, Value<'a, CONST>);
    type Iterator<'b> = std::slice::Iter<'b, (Name<'a>, Value<'a, CONST>)> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields.iter()
    }
}

impl bluejay_core::IntegerValue for IntValue {
    fn to_i32(&self) -> i32 {
        self.value()
    }
}

impl bluejay_core::FloatValue for FloatValue {
    fn to_f64(&self) -> f64 {
        self.value()
    }
}

impl bluejay_core::StringValue for StringValue {}

impl<'a> bluejay_core::EnumValue for Name<'a> {}
