use crate::Span;
use enum_as_inner::EnumAsInner;

mod float_value;
mod has_span;
mod int_value;
mod name;
mod punctuator;
mod string_value;
pub use float_value::FloatValue;
pub use has_span::HasSpan;
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
    StringValue(StringValue),
}

impl<'a> HasSpan for LexicalToken<'a> {
    fn span(&self) -> Span {
        match self {
            Self::FloatValue(f) => f.span(),
            Self::IntValue(i) => i.span(),
            Self::StringValue(s) => s.span(),
            Self::Name(n) => n.span(),
            Self::Punctuator(p) => p.span(),
        }
    }
}
