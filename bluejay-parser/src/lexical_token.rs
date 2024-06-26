use crate::{HasSpan, Span};
use enum_as_inner::EnumAsInner;

mod bool_value;
mod float_value;
mod int_value;
mod name;
mod punctuator;
mod string_value;
pub use bool_value::BooleanValue;
pub use float_value::FloatValue;
pub use int_value::IntValue;
pub use name::Name;
pub use punctuator::{Punctuator, PunctuatorType};
pub use string_value::StringValue;

#[derive(PartialEq, Debug, EnumAsInner)]
pub enum LexicalToken<'a> {
    Punctuator(Punctuator),
    Name(Name<'a>),
    IntValue(IntValue),
    FloatValue(FloatValue),
    StringValue(StringValue<'a>),
    BooleanValue(BooleanValue),
}

impl<'a> HasSpan for LexicalToken<'a> {
    fn span(&self) -> &Span {
        match self {
            Self::FloatValue(f) => f.span(),
            Self::IntValue(i) => i.span(),
            Self::StringValue(s) => s.span(),
            Self::Name(n) => n.span(),
            Self::Punctuator(p) => p.span(),
            Self::BooleanValue(p) => p.span(),
        }
    }
}

impl<'a> From<LexicalToken<'a>> for Span {
    fn from(value: LexicalToken<'a>) -> Self {
        match value {
            LexicalToken::FloatValue(f) => f.into(),
            LexicalToken::IntValue(i) => i.into(),
            LexicalToken::StringValue(s) => s.into(),
            LexicalToken::Name(n) => n.into(),
            LexicalToken::Punctuator(p) => p.into(),
            LexicalToken::BooleanValue(p) => p.into(),
        }
    }
}
