use std::num::{ParseIntError, ParseFloatError};

use logos::{Logos, Lexer, FilterResult};
use crate::scanner::{ScanError, Scanner};
use crate::lexical_token::{LexicalToken, Punctuator, PunctuatorType, Name, IntValue, FloatValue, StringValue};
use crate::Span;

mod block_string_scanner;
mod string_scanner;

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'a> {
    // Punctuators
    #[token("!")]
    Bang,
    #[token("$")]
    Dollar,
    #[token("&")]
    Ampersand,
    #[token("(")]
    OpenRoundBracket,
    #[token(")")]
    CloseRoundBracket,
    #[token("...")]
    Ellipse,
    #[token(":")]
    Colon,
    #[token("=")]
    Equals,
    #[token("@")]
    At,
    #[token("[")]
    OpenSquareBracket,
    #[token("]")]
    CloseSquareBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("|")]
    Pipe,

    // Name
    #[regex(r"[a-zA-Z_]\w*")]
    Name(&'a str),

    // IntValue
    #[regex(r"-?(?:0|[1-9]\d*)", parse_integer)]
    IntValue(Result<i32, ParseIntError>),

    // FloatValue
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+[eE][+-]?\d+|\.\d+|[eE][+-]?\d+)", parse_float)]
    FloatValue(Result<f64, ParseFloatError>),

    // StringValue
    #[regex(r#""(?:[\u0009\u0020\u0021\u0023-\u005B\u005D-\uFFFF]|\\u[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]|\\["\\/bfnrt])*""#r, parse_string)]
    StringValue(String),

    #[regex("\"\"\"", parse_block_string)]
    BlockStringValue(String),

    // Skippable
    #[error]
    #[regex(r"[\uFEFF\u0009\u0020\u000D\u000A,]+", logos::skip)]
    #[regex(r"#[\u0009\u0020-\uFFFF]*", logos::skip)] // comments
    Error,
}

fn parse_block_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> FilterResult<String> {
    match block_string_scanner::Token::parse(lexer.remainder()) {
        Ok((s, bytes_consumed)) => {
            lexer.bump(bytes_consumed);
            FilterResult::Emit(s)
        },
        Err(_) => {
            lexer.bump(lexer.remainder().len());
            FilterResult::Error
        },
    }
}

fn parse_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> String {
    string_scanner::Token::parse(lexer.slice())
}

fn validate_number<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> bool {
    let invalid_trail_bytes = lexer.remainder().chars().position(|c| {
        !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))
    }).unwrap_or(lexer.remainder().len());

    lexer.bump(invalid_trail_bytes);

    invalid_trail_bytes == 0
}

fn parse_integer<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> FilterResult<Result<i32, ParseIntError>> {
    if validate_number(lexer) {
        FilterResult::Emit(lexer.slice().parse())
    } else {
        FilterResult::Error
    }
}

fn parse_float<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> FilterResult<Result<f64, ParseFloatError>> {
    if validate_number(lexer) {
        FilterResult::Emit(lexer.slice().parse())
    } else {
        FilterResult::Error
    }
}

#[repr(transparent)]
pub struct LogosScanner<'a>(Lexer<'a, Token<'a>>);

impl<'a> Iterator for LogosScanner<'a> {
    type Item = Result<LexicalToken<'a>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|token| {
            let span = Span::new(self.0.span());
            match token {
                Token::Bang => punctuator(PunctuatorType::Bang, span),
                Token::Dollar => punctuator(PunctuatorType::Dollar, span),
                Token::Ampersand => punctuator(PunctuatorType::Ampersand, span),
                Token::OpenRoundBracket => punctuator(PunctuatorType::OpenRoundBracket, span),
                Token::CloseRoundBracket => punctuator(PunctuatorType::CloseRoundBracket, span),
                Token::Ellipse => punctuator(PunctuatorType::Ellipse, span),
                Token::Colon => punctuator(PunctuatorType::Colon, span),
                Token::Equals => punctuator(PunctuatorType::Equals, span),
                Token::At => punctuator(PunctuatorType::At, span),
                Token::OpenSquareBracket => punctuator(PunctuatorType::OpenSquareBracket, span),
                Token::CloseSquareBracket => punctuator(PunctuatorType::CloseSquareBracket, span),
                Token::OpenBrace => punctuator(PunctuatorType::OpenBrace, span),
                Token::CloseBrace => punctuator(PunctuatorType::CloseBrace, span),
                Token::Pipe => punctuator(PunctuatorType::Pipe, span),
                Token::Name(s) => Ok(LexicalToken::Name(Name::new(s, span))),
                Token::IntValue(res) => match res {
                    Ok(val) => Ok(LexicalToken::IntValue(IntValue::new(val, span))),
                    Err(_) => Err(ScanError::IntegerValueTooLarge(span)),
                },
                Token::FloatValue(res) => match res {
                    Ok(val) => Ok(LexicalToken::FloatValue(FloatValue::new(val, span))),
                    Err(_) => Err(ScanError::FloatValueTooLarge(span)),
                },
                Token::StringValue(s) => Ok(LexicalToken::StringValue(StringValue::new(s, span))),
                Token::BlockStringValue(s) => Ok(LexicalToken::StringValue(StringValue::new(s, span))),
                Token::Error => Err(ScanError::UnrecognizedTokenError(span)),
            }
        })
    }
}

fn punctuator<'a>(pt: PunctuatorType, span: Span) -> Result<LexicalToken<'a>, ScanError> {
    Ok(LexicalToken::Punctuator(Punctuator::new(pt, span)))
}

impl<'a> Scanner<'a> for LogosScanner<'a> {
    fn empty_span(&self) -> Span {
        let n = self.0.span().start;
        Span::new(n..n)
    }
}

impl<'a> LogosScanner<'a> {
    pub fn new(s: &'a <Token<'a> as Logos<'a>>::Source) -> Self {
        Self(Token::lexer(s))
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, Logos};

    #[test]
    fn block_string_test() {
        let s = r#"
            """
                This is my multiline string!

                Isn't it cool?
            """
        "#;

        let mut lexer = Token::lexer(s);

        assert_eq!(
            Some(Token::BlockStringValue(String::from("This is my multiline string!\n\nIsn't it cool?"))),
            lexer.next(),
        );

        assert_eq!(
            13..109,
            lexer.span(),
        )
    }

    #[test]
    fn string_test() {
        let s = "\"This is a string with escaped characters and unicode: \\uABCD!\\n\"";

        let mut lexer = Token::lexer(s);

        assert_eq!(
            Some(Token::StringValue(String::from("This is a string with escaped characters and unicode: \u{ABCD}!\n"))),
            lexer.next(),
        )
    }

    #[test]
    fn int_test() {
        let s = "12345";

        let mut lexer = Token::lexer(s);

        assert_eq!(Some(Token::IntValue(Ok(12345))), lexer.next());
    }

    #[test]
    fn float_test() {
        let s = "12345.6789e123";

        let mut lexer = Token::lexer(s);

        assert_eq!(Some(Token::FloatValue(Ok(12345.6789e123))), lexer.next());
    }
}
