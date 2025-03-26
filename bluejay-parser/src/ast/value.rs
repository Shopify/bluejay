use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::{
    FloatValue, IntValue, LexicalToken, Name, PunctuatorType, StringValue, Variable,
};
use crate::{HasSpan, Span};
use bluejay_core::{
    AsIter, ListValue as CoreListValue, ObjectValue as CoreObjectValue, Value as CoreValue,
    ValueReference,
};

#[derive(Debug)]
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

impl<'a, const CONST: bool> CoreValue<CONST> for Value<'a, CONST> {
    type List = ListValue<'a, CONST>;
    type Object = ObjectValue<'a, CONST>;
    type Variable = Variable<'a>;

    fn as_ref(&self) -> ValueReference<'_, CONST, Self> {
        match self {
            Self::Variable(v) => ValueReference::Variable(v),
            Self::Integer(i) => ValueReference::Integer(i.value()),
            Self::Float(f) => ValueReference::Float(f.value()),
            Self::String(s) => ValueReference::String(s.as_ref()),
            Self::Boolean(b) => ValueReference::Boolean(b.value()),
            Self::Null(_) => ValueReference::Null,
            Self::Enum(e) => ValueReference::Enum(e.as_ref()),
            Self::List(l) => ValueReference::List(l),
            Self::Object(o) => ValueReference::Object(o),
        }
    }
}

impl<'a, const CONST: bool> FromTokens<'a> for Value<'a, CONST> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        match tokens.next() {
            Some(LexicalToken::IntValue(int)) => Ok(Self::Integer(int)),
            Some(LexicalToken::FloatValue(float)) => Ok(Self::Float(float)),
            Some(LexicalToken::StringValue(string)) => Ok(Self::String(string)),
            Some(LexicalToken::Name(name)) => Ok(match name.as_str() {
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
            }),
            Some(LexicalToken::VariableName(name)) => {
                if CONST {
                    Err(ParseError::UnexpectedToken { span: name.into() })
                } else {
                    Ok(Self::Variable(name))
                }
            }
            Some(LexicalToken::Punctuator(p))
                if p.r#type() == PunctuatorType::OpenSquareBracket =>
            {
                let open_span = p.span().clone();
                let mut list: Vec<Self> = Vec::new();
                let close_span = loop {
                    if let Some(close_span) =
                        tokens.next_if_punctuator(PunctuatorType::CloseSquareBracket)
                    {
                        break close_span;
                    }
                    list.push(Self::from_tokens(tokens, depth_limiter.bump()?)?);
                };
                let span = open_span.merge(&close_span);
                Ok(Self::List(ListValue {
                    elements: list,
                    span,
                }))
            }
            Some(LexicalToken::Punctuator(p)) if p.r#type() == PunctuatorType::OpenBrace => {
                let open_span = p.span().clone();
                let mut object: Vec<_> = Vec::new();
                let close_span = loop {
                    if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace)
                    {
                        break close_span;
                    }
                    let name = tokens.expect_name()?;
                    tokens.expect_punctuator(PunctuatorType::Colon)?;
                    let value = Self::from_tokens(tokens, depth_limiter.bump()?)?;
                    object.push((name, value));
                };
                let span = open_span.merge(&close_span);
                Ok(Self::Object(ObjectValue {
                    fields: object,
                    span,
                }))
            }
            token => Err(token
                .map(|token| ParseError::UnexpectedToken { span: token.into() })
                .unwrap_or_else(|| tokens.unexpected_eof())),
        }
    }
}

impl<const CONST: bool> HasSpan for Value<'_, CONST> {
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
    type Iterator<'b>
        = std::slice::Iter<'b, Self::Item>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.elements.iter()
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
    type Iterator<'b>
        = std::iter::Map<
        std::slice::Iter<'b, (Name<'a>, Value<'a, CONST>)>,
        fn(&'b (Name<'a>, Value<'a, CONST>)) -> (&'b Name<'a>, &'b Value<'a, CONST>),
    >
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields.iter().map(|(key, value)| (key, value))
    }
}

impl<'a, const CONST: bool> AsIter for ObjectValue<'a, CONST> {
    type Item = (Name<'a>, Value<'a, CONST>);
    type Iterator<'b>
        = std::slice::Iter<'b, (Name<'a>, Value<'a, CONST>)>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields.iter()
    }
}

impl<const CONST: bool> HasSpan for ObjectValue<'_, CONST> {
    fn span(&self) -> &Span {
        &self.span
    }
}
