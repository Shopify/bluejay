use crate::ast::{FromTokens, ParseError, Tokens, TryFromTokens, Variable};
use crate::lexical_token::{FloatValue, IntValue, Name, PunctuatorType, StringValue};
use crate::{HasSpan, Span};
use bluejay_core::{
    AbstractValue, AsIter, ListValue as CoreListValue, ObjectValue as CoreObjectValue,
    Value as CoreValue, ValueFromAbstract,
};

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Value<'a, const CONST: bool> {
    Variable(Variable<'a>),
    Integer(IntValue),
    Float(FloatValue),
    String(StringValue<'a>),
    Boolean(BooleanValue),
    Null(Name<'a>),
    Enum(Name<'a>),
    List(ListValue<'a, CONST>),
    Object(ObjectValue<'a, CONST>),
}

impl<'a, const CONST: bool> AbstractValue<CONST> for Value<'a, CONST> {
    type List = ListValue<'a, CONST>;
    type Object = ObjectValue<'a, CONST>;
    type Variable = Variable<'a>;

    fn as_ref(&self) -> ValueFromAbstract<'_, CONST, Self> {
        match self {
            Self::Variable(v) => CoreValue::Variable(v),
            Self::Integer(i) => CoreValue::Integer(i.value()),
            Self::Float(f) => CoreValue::Float(f.value()),
            Self::String(s) => CoreValue::String(s.as_ref()),
            Self::Boolean(b) => CoreValue::Boolean(b.value()),
            Self::Null(_) => CoreValue::Null,
            Self::Enum(e) => CoreValue::Enum(e.as_ref()),
            Self::List(l) => CoreValue::List(l),
            Self::Object(o) => CoreValue::Object(o),
        }
    }
}

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
                                span: name.into(),
                            }),
                            "false" => Self::Boolean(BooleanValue {
                                value: false,
                                span: name.into(),
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
                .map(|token| ParseError::UnexpectedToken { span: token.into() })
                .unwrap_or_else(|| tokens.unexpected_eof()))
        })
    }
}

impl<'a, const CONST: bool> HasSpan for Value<'a, CONST> {
    fn span(&self) -> &Span {
        match self {
            Self::Boolean(b) => &b.span,
            Self::Enum(e) => e.span(),
            Self::Float(f) => f.span(),
            Self::Integer(i) => i.span(),
            Self::List(l) => &l.span,
            Self::Null(n) => n.span(),
            Self::Object(o) => &o.span,
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

impl BooleanValue {
    fn value(&self) -> bool {
        self.value
    }
}

#[derive(Debug)]
pub struct ListValue<'a, const CONST: bool> {
    elements: Vec<Value<'a, CONST>>,
    span: Span,
}

impl<'a, const CONST: bool> CoreListValue<CONST> for ListValue<'a, CONST> {
    type Value = Value<'a, CONST>;
}

impl<'a, const CONST: bool> AsIter for ListValue<'a, CONST> {
    type Item = Value<'a, CONST>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.elements.iter()
    }
}

impl<'a, const CONST: bool> AsRef<[Value<'a, CONST>]> for ListValue<'a, CONST> {
    fn as_ref(&self) -> &[Value<'a, CONST>] {
        self.elements.as_slice()
    }
}

#[derive(Debug)]
pub struct ObjectValue<'a, const CONST: bool> {
    fields: Vec<(Name<'a>, Value<'a, CONST>)>,
    span: Span,
}

impl<'a, const CONST: bool> CoreObjectValue<CONST> for ObjectValue<'a, CONST> {
    type Key = Name<'a>;
    type Value = Value<'a, CONST>;
    type Iterator<'b> = std::iter::Map<std::slice::Iter<'b, (Name<'a>, Value<'a, CONST>)>, fn(&'b (Name<'a>, Value<'a, CONST>)) -> (&'b Name<'a>, &'b Value<'a, CONST>)> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields.iter().map(|(key, value)| (key, value))
    }
}

impl<'a, const CONST: bool> AsIter for ObjectValue<'a, CONST> {
    type Item = (Name<'a>, Value<'a, CONST>);
    type Iterator<'b> = std::slice::Iter<'b, (Name<'a>, Value<'a, CONST>)> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields.iter()
    }
}

impl<'a, const CONST: bool> HasSpan for ObjectValue<'a, CONST> {
    fn span(&self) -> &Span {
        &self.span
    }
}
