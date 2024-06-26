use crate::lexical_token::LexicalToken;
use crate::Span;
mod lex_error;
mod logos_lexer;
pub use lex_error::{LexError, StringValueLexError};
pub use logos_lexer::LogosLexer;

pub trait Lexer<'a>: Iterator<Item = Result<LexicalToken<'a>, (LexError, Span)>> {
    fn empty_span(&self) -> Span;
}
