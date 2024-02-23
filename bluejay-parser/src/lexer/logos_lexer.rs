use std::num::{ParseFloatError, ParseIntError};

use crate::lexer::{LexError, Lexer};
use crate::lexical_token::{
    FloatValue, IntValue, LexicalToken, Name, Punctuator, PunctuatorType, StringValue,
};
use crate::Span;
use logos::Logos;
use std::borrow::Cow;

mod block_string_lexer;
mod string_lexer;

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern intpart = r"-?(?:0|[1-9]\d*)")]
#[logos(subpattern decimalpart = r"\.\d+")]
#[logos(subpattern exponentpart = r"[eE][+-]?\d+")]
#[logos(subpattern hexdigit = r"[0-9A-Fa-f]")]
#[logos(subpattern fixedunicode = r"\\u[0-9A-Fa-f]{4}")]
#[logos(skip r"[\uFEFF\t \n\r,]+")]
#[logos(skip r"#[^\n\r]*")] // comments
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
    #[regex(r"(?&intpart)", parse_integer)]
    IntValue(Result<i32, ParseIntError>),

    // FloatValue
    #[regex(
        r"(?&intpart)(?:(?&decimalpart)(?&exponentpart)|(?&decimalpart)|(?&exponentpart))",
        parse_float
    )]
    FloatValue(Result<f64, ParseFloatError>),

    // StringValue
    #[regex(r#""(?:[^\\"\n\r]|(?&fixedunicode)|\\u\{(?&hexdigit)+\}|\\["\\/bfnrt])*""#r, parse_string)]
    StringValue(Result<Cow<'a, str>, Vec<Span>>),

    #[token("\"\"\"", parse_block_string)]
    BlockStringValue(Cow<'a, str>),
}

fn parse_block_string<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> Option<Cow<'a, str>> {
    match block_string_lexer::Token::parse(lexer.remainder()) {
        Ok((s, bytes_consumed)) => {
            lexer.bump(bytes_consumed);
            Some(s)
        }
        Err(_) => {
            lexer.bump(lexer.remainder().len());
            None
        }
    }
}

fn parse_string<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> Result<Cow<'a, str>, Vec<Span>> {
    string_lexer::Token::parse(lexer.slice(), lexer.span().start)
}

fn validate_number<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> bool {
    let invalid_trail_bytes = lexer
        .remainder()
        .chars()
        .position(|c| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.')))
        .unwrap_or(lexer.remainder().len());

    lexer.bump(invalid_trail_bytes);

    invalid_trail_bytes == 0
}

fn parse_integer<'a>(
    lexer: &mut logos::Lexer<'a, Token<'a>>,
) -> Option<Result<i32, ParseIntError>> {
    validate_number(lexer).then(|| lexer.slice().parse())
}

fn parse_float<'a>(
    lexer: &mut logos::Lexer<'a, Token<'a>>,
) -> Option<Result<f64, ParseFloatError>> {
    validate_number(lexer).then(|| lexer.slice().parse())
}

#[repr(transparent)]
pub struct LogosLexer<'a>(logos::Lexer<'a, Token<'a>>);

impl<'a> Iterator for LogosLexer<'a> {
    type Item = Result<LexicalToken<'a>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|result| {
            let span = Span::new(self.0.span());
            if let Ok(token) = result {
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
                    Token::CloseSquareBracket => {
                        punctuator(PunctuatorType::CloseSquareBracket, span)
                    }
                    Token::OpenBrace => punctuator(PunctuatorType::OpenBrace, span),
                    Token::CloseBrace => punctuator(PunctuatorType::CloseBrace, span),
                    Token::Pipe => punctuator(PunctuatorType::Pipe, span),
                    Token::Name(s) => Ok(LexicalToken::Name(Name::new(s, span))),
                    Token::IntValue(res) => match res {
                        Ok(val) => Ok(LexicalToken::IntValue(IntValue::new(val, span))),
                        Err(_) => Err(LexError::IntegerValueTooLarge(span)),
                    },
                    Token::FloatValue(res) => match res {
                        Ok(val) => Ok(LexicalToken::FloatValue(FloatValue::new(val, span))),
                        Err(_) => Err(LexError::FloatValueTooLarge(span)),
                    },
                    Token::StringValue(res) => res
                        .map(|s| LexicalToken::StringValue(StringValue::new(s, span)))
                        .map_err(LexError::StringWithInvalidEscapedUnicode),
                    Token::BlockStringValue(s) => {
                        Ok(LexicalToken::StringValue(StringValue::new(s, span)))
                    }
                }
            } else {
                Err(LexError::UnrecognizedTokenError(span))
            }
        })
    }
}

fn punctuator<'a>(pt: PunctuatorType, span: Span) -> Result<LexicalToken<'a>, LexError> {
    Ok(LexicalToken::Punctuator(Punctuator::new(pt, span)))
}

impl<'a> Lexer<'a> for LogosLexer<'a> {
    fn empty_span(&self) -> Span {
        let n = self.0.span().start;
        Span::new(n..n)
    }
}

impl<'a> LogosLexer<'a> {
    pub fn new(s: &'a <Token<'a> as Logos<'a>>::Source) -> Self {
        Self(Token::lexer(s))
    }
}

#[cfg(test)]
mod tests {
    use super::{Logos, Span, Token};

    #[test]
    fn block_string_test() {
        assert_eq!(
            Some(Ok(Token::BlockStringValue(
                "This is my multiline string!\n\nIsn't it cool? 🔥".into()
            ))),
            Token::lexer(
                r#"
                    """
                        This is my multiline string!

                        Isn't it cool? 🔥
                    """
                "#
            )
            .next(),
        );
        assert_eq!(
            Some((Ok(Token::BlockStringValue("Testing span".into())), 1..19,)),
            Token::lexer(r#" """Testing span""" "#).spanned().next(),
        );
        assert_eq!(
            Some(Ok(Token::BlockStringValue(
                "Testing escaped block quote \"\"\"".into()
            ))),
            Token::lexer(r#" """Testing escaped block quote \"""""" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::BlockStringValue(
                "Testing \n various \n newlines".into()
            ))),
            Token::lexer("\"\"\"\nTesting \r various \r\n newlines\"\"\"").next(),
        );
        assert_eq!(
            Some(Err(())),
            Token::lexer(r#" """This is a block string that doesn't end "#).next(),
        );
        assert_eq!(
            vec![
                Ok(Token::BlockStringValue("".into())),
                Ok(Token::StringValue(Ok("".into()))),
            ],
            Token::lexer(r#" """""""" "#).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn string_test() {
        assert_eq!(
            Some(Ok(Token::StringValue(Ok(
                "This is a string with escaped characters and unicode: 🥳\u{ABCD}\u{10FFFF}!\n"
                    .into()
            )))),
            Token::lexer("\"This is a string with escaped characters and unicode: 🥳\\uABCD\\u{10FFFF}!\\n\"").next(),
        );
        assert_eq!(
            Some(Err(())),
            Token::lexer("\"This is a string with a newline \n Not allowed!\"").next(),
        );
        assert_eq!(
            Some((Ok(Token::StringValue(Ok("Testing span".into()))), 1..15,)),
            Token::lexer(r#" "Testing span" "#).spanned().next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Err(vec![Span::from(2..8)])))),
            Token::lexer(r#" "\uD800" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Err(vec![Span::from(2..12)])))),
            Token::lexer(r#" "\u{00D800}" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Ok("🔥".into())))),
            Token::lexer(r#" "\uD83D\uDD25" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Ok("\u{1234}\u{ABCD}".into())))),
            Token::lexer(r#" "\u1234\uABCD" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Err(vec![Span::from(2..8)])))),
            Token::lexer(r#" "\uDEAD\uDEAD" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Err(vec![Span::from(8..14)])))),
            Token::lexer(r#" "\uD800\uD800" "#).next(),
        );
        assert_eq!(
            Some(Err(())),
            Token::lexer(r#" "This is a string that doesn't end "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue(Err(vec![Span::from(2..15)])))),
            Token::lexer(r#" "\u{100000000}" "#).next(),
        );
    }

    #[test]
    fn int_test() {
        assert_eq!(
            Some(Ok(Token::IntValue(Ok(12345)))),
            Token::lexer("12345").next()
        );
        assert_eq!(Some(Err(())), Token::lexer("012345").next(),);
        assert_eq!(
            Some((Err(()), 0..6)),
            Token::lexer("12345A").spanned().next()
        );
        assert_eq!(
            Some((Err(()), 0..6)),
            Token::lexer("12345_").spanned().next()
        );
        assert_eq!(Some(Ok(Token::IntValue(Ok(0)))), Token::lexer("0").next());
        assert_eq!(Some(Ok(Token::IntValue(Ok(0)))), Token::lexer("-0").next());
        let int_too_positive = (i64::from(i32::MAX) + 1).to_string();
        assert_eq!(
            Token::lexer(&int_too_positive).next(),
            Some(Ok(Token::IntValue(Err(int_too_positive
                .parse::<i32>()
                .unwrap_err()))))
        );
        let int_too_negative = (i64::from(i32::MIN) - 1).to_string();
        assert_eq!(
            Token::lexer(&int_too_negative).next(),
            Some(Ok(Token::IntValue(Err(int_too_negative
                .parse::<i32>()
                .unwrap_err()))))
        );
    }

    #[test]
    fn float_test() {
        assert_eq!(
            Some(Ok(Token::FloatValue(Ok(12345.6789e123)))),
            Token::lexer("12345.6789e123").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(Ok(12345e123)))),
            Token::lexer("12345e123").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(Ok(12345.6789)))),
            Token::lexer("12345.6789").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(Ok(0.0)))),
            Token::lexer("0.00000000").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(Ok(-1.23)))),
            Token::lexer("-1.23").next()
        );
        assert_eq!(Some(Err(())), Token::lexer("012345.6789e123").next());
        assert_eq!(Some(Err(())), Token::lexer("-012345.6789e123").next());
        assert_eq!(Some(Err(())), Token::lexer("1.").next());
        assert_eq!(
            Some((Err(()), 0..15)),
            Token::lexer("12345.6789e123A").spanned().next()
        );
    }

    #[test]
    fn name_test() {
        assert_eq!(Some(Ok(Token::Name("name"))), Token::lexer("name").next());
        assert_eq!(
            Some(Ok(Token::Name("__name"))),
            Token::lexer("__name").next()
        );
        assert_eq!(Some(Ok(Token::Name("name1"))), Token::lexer("name1").next());
        assert_eq!(Some(Err(())), Token::lexer("1name").next());
        assert_eq!(
            vec![Ok(Token::Name("dashed")), Err(()), Ok(Token::Name("name"))],
            Token::lexer("dashed-name").collect::<Vec<_>>(),
        );
    }

    #[test]
    fn comment_test() {
        assert_eq!(None, Token::lexer("# this is a comment").next());
        assert_eq!(
            Some(Ok(Token::Ampersand)),
            Token::lexer("# this is a comment\n# this is another comment\r&").next(),
        );
    }
}
