use super::HasSpan;
use crate::Span;
use std::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PunctuatorType {
    Bang,
    Ampersand,
    OpenRoundBracket,
    CloseRoundBracket,
    Ellipse,
    Colon,
    Equals,
    At,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenBrace,
    Pipe,
    CloseBrace,
}

#[derive(PartialEq, Debug)]
pub struct Punctuator {
    r#type: PunctuatorType,
    span: Span,
}

impl Punctuator {
    pub fn r#type(&self) -> PunctuatorType {
        self.r#type
    }

    pub(crate) fn new(r#type: PunctuatorType, span: Span) -> Self {
        Self { r#type, span }
    }
}

impl HasSpan for Punctuator {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl From<Punctuator> for Span {
    fn from(value: Punctuator) -> Self {
        value.span
    }
}

impl fmt::Display for PunctuatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Bang => "!",
            Self::Ampersand => "&",
            Self::OpenRoundBracket => "(",
            Self::CloseRoundBracket => ")",
            Self::Colon => ":",
            Self::Equals => "=",
            Self::At => "@",
            Self::OpenSquareBracket => "[",
            Self::CloseSquareBracket => "]",
            Self::OpenBrace => "{",
            Self::Pipe => "|",
            Self::CloseBrace => "}",
            Self::Ellipse => "...",
        };
        write!(f, "{s}")
    }
}
