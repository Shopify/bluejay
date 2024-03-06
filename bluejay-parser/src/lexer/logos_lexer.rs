use crate::lexer::{LexError, Lexer};
use crate::lexical_token::{
    FloatValue, IntValue, LexicalToken, Name, Punctuator, PunctuatorType, StringValue,
};
use crate::Span;
use logos::Logos;
use std::borrow::Cow;

mod block_string_lexer;
mod string_lexer;

#[derive(Default)]
pub(crate) struct Extras {
    graphql_ruby_compatibility: bool,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern intpart = r"-?(?:0|[1-9]\d*)")]
#[logos(subpattern decimalpart = r"\.\d+")]
#[logos(subpattern exponentpart = r"[eE][+-]?\d+")]
#[logos(subpattern hexdigit = r"[0-9A-Fa-f]")]
#[logos(subpattern fixedunicode = r"\\u[0-9A-Fa-f]{4}")]
#[logos(error = LexError)]
#[logos(skip r"[\uFEFF\t \n\r,]+")]
#[logos(skip r"#[^\n\r]*")] // comments
#[logos(extras = Extras)]
pub(crate) enum Token<'a> {
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
    IntValue(i32),

    // FloatValue
    #[regex(
        r"(?&intpart)(?:(?&decimalpart)(?&exponentpart)|(?&decimalpart)|(?&exponentpart))",
        parse_float
    )]
    FloatValue(f64),

    // StringValue
    #[token("\"", string_lexer::Token::parse)]
    StringValue(Cow<'a, str>),

    #[token("\"\"\"", block_string_lexer::Token::parse)]
    BlockStringValue(Cow<'a, str>),
}

fn validate_number_no_trailing_name_start<'a>(
    lexer: &mut logos::Lexer<'a, Token<'a>>,
) -> Result<(), LexError> {
    if lexer.extras.graphql_ruby_compatibility {
        return Ok(());
    }

    let invalid_trail_bytes = lexer
        .remainder()
        .chars()
        .position(|c| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.')))
        .unwrap_or(lexer.remainder().len());

    lexer.bump(invalid_trail_bytes);

    if invalid_trail_bytes == 0 {
        Ok(())
    } else {
        Err(LexError::UnrecognizedToken)
    }
}

fn parse_integer<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> Result<i32, LexError> {
    validate_number_no_trailing_name_start(lexer).and_then(|_| {
        lexer
            .slice()
            .parse()
            .map_err(|_| LexError::IntegerValueTooLarge)
    })
}

fn parse_float<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> Result<f64, LexError> {
    validate_number_no_trailing_name_start(lexer).and_then(|_| {
        lexer
            .slice()
            .parse()
            .map_err(|_| LexError::FloatValueTooLarge)
    })
}

#[repr(transparent)]
pub struct LogosLexer<'a>(logos::Lexer<'a, Token<'a>>);

impl<'a> Iterator for LogosLexer<'a> {
    type Item = Result<LexicalToken<'a>, (LexError, Span)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|result| {
            result
                .map(|token| {
                    let span = Span::new(self.0.span());
                    match token {
                        Token::Bang => punctuator(PunctuatorType::Bang, span),
                        Token::Dollar => punctuator(PunctuatorType::Dollar, span),
                        Token::Ampersand => punctuator(PunctuatorType::Ampersand, span),
                        Token::OpenRoundBracket => {
                            punctuator(PunctuatorType::OpenRoundBracket, span)
                        }
                        Token::CloseRoundBracket => {
                            punctuator(PunctuatorType::CloseRoundBracket, span)
                        }
                        Token::Ellipse => punctuator(PunctuatorType::Ellipse, span),
                        Token::Colon => punctuator(PunctuatorType::Colon, span),
                        Token::Equals => punctuator(PunctuatorType::Equals, span),
                        Token::At => punctuator(PunctuatorType::At, span),
                        Token::OpenSquareBracket => {
                            punctuator(PunctuatorType::OpenSquareBracket, span)
                        }
                        Token::CloseSquareBracket => {
                            punctuator(PunctuatorType::CloseSquareBracket, span)
                        }
                        Token::OpenBrace => punctuator(PunctuatorType::OpenBrace, span),
                        Token::CloseBrace => punctuator(PunctuatorType::CloseBrace, span),
                        Token::Pipe => punctuator(PunctuatorType::Pipe, span),
                        Token::Name(s) => LexicalToken::Name(Name::new(s, span)),
                        Token::IntValue(val) => LexicalToken::IntValue(IntValue::new(val, span)),
                        Token::FloatValue(val) => {
                            LexicalToken::FloatValue(FloatValue::new(val, span))
                        }
                        Token::StringValue(val) => {
                            LexicalToken::StringValue(StringValue::new(val, span))
                        }
                        Token::BlockStringValue(val) => {
                            LexicalToken::StringValue(StringValue::new(val, span))
                        }
                    }
                })
                .map_err(|err| (err, Span::new(self.0.span())))
        })
    }
}

fn punctuator<'a>(pt: PunctuatorType, span: Span) -> LexicalToken<'a> {
    LexicalToken::Punctuator(Punctuator::new(pt, span))
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

    pub fn with_graphql_ruby_compatibility(mut self, enabled: bool) -> Self {
        self.0.extras.graphql_ruby_compatibility = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Extras, Token};
    use crate::lexer::{LexError, Span, StringValueLexError};
    use logos::Logos;

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
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer(r#" """This is a block string that doesn't end "#).next(),
        );
        assert_eq!(
            vec![
                Ok(Token::BlockStringValue("".into())),
                Ok(Token::StringValue("".into())),
            ],
            Token::lexer(r#" """""""" "#).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn string_test() {
        assert_eq!(
            Some(Ok(Token::StringValue(
                "This is a string with escaped characters and unicode: 🥳\u{ABCD}\u{10FFFF}!\n"
                    .into()
            ))),
            Token::lexer("\"This is a string with escaped characters and unicode: 🥳\\uABCD\\u{10FFFF}!\\n\"").next(),
        );
        assert_eq!(
            Some(Err(LexError::StringValueInvalid(vec![
                StringValueLexError::InvalidCharacters(Span::from(33..34))
            ]))),
            Token::lexer("\"This is a string with a newline \n Not allowed!\"").next(),
        );
        assert_eq!(
            Some((Ok(Token::StringValue("Testing span".into())), 1..15,)),
            Token::lexer(r#" "Testing span" "#).spanned().next(),
        );
        assert_eq!(
            Some(Err(LexError::StringValueInvalid(vec![
                StringValueLexError::InvalidUnicodeEscapeSequence(Span::from(2..8))
            ]))),
            Token::lexer(r#" "\uD800" "#).next(),
        );
        assert_eq!(
            Some(Err(LexError::StringValueInvalid(vec![
                StringValueLexError::InvalidUnicodeEscapeSequence(Span::from(2..12))
            ]))),
            Token::lexer(r#" "\u{00D800}" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue("🔥".into()))),
            Token::lexer(r#" "\uD83D\uDD25" "#).next(),
        );
        assert_eq!(
            Some(Ok(Token::StringValue("\u{1234}\u{ABCD}".into()))),
            Token::lexer(r#" "\u1234\uABCD" "#).next(),
        );
        assert_eq!(
            Some(Err(LexError::StringValueInvalid(vec![
                StringValueLexError::InvalidUnicodeEscapeSequence(Span::from(2..8))
            ]))),
            Token::lexer(r#" "\uDEAD\uDEAD" "#).next(),
        );
        assert_eq!(
            Some(Err(LexError::StringValueInvalid(vec![
                StringValueLexError::InvalidUnicodeEscapeSequence(Span::from(8..14))
            ]))),
            Token::lexer(r#" "\uD800\uD800" "#).next(),
        );
        assert_eq!(
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer(r#" "This is a string that doesn't end "#).next(),
        );
        assert_eq!(
            Some(Err(LexError::StringValueInvalid(vec![
                StringValueLexError::InvalidUnicodeEscapeSequence(Span::from(2..15))
            ]))),
            Token::lexer(r#" "\u{100000000}" "#).next(),
        );
    }

    #[test]
    fn int_test() {
        assert_eq!(
            Some(Ok(Token::IntValue(12345))),
            Token::lexer("12345").next()
        );
        assert_eq!(
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer("012345").next(),
        );
        assert_eq!(
            Some((Err(LexError::UnrecognizedToken), 0..6)),
            Token::lexer("12345A").spanned().next()
        );
        assert_eq!(
            Some((Err(LexError::UnrecognizedToken), 0..6)),
            Token::lexer("12345_").spanned().next()
        );
        assert_eq!(Some(Ok(Token::IntValue(0))), Token::lexer("0").next());
        assert_eq!(Some(Ok(Token::IntValue(0))), Token::lexer("-0").next());
        let int_too_positive = (i64::from(i32::MAX) + 1).to_string();
        assert_eq!(
            Token::lexer(&int_too_positive).next(),
            Some(Err(LexError::IntegerValueTooLarge))
        );
        let int_too_negative = (i64::from(i32::MIN) - 1).to_string();
        assert_eq!(
            Token::lexer(&int_too_negative).next(),
            Some(Err(LexError::IntegerValueTooLarge))
        );
    }

    #[test]
    fn float_test() {
        assert_eq!(
            Some(Ok(Token::FloatValue(12345.6789e123))),
            Token::lexer("12345.6789e123").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(12345e123))),
            Token::lexer("12345e123").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(12345.6789))),
            Token::lexer("12345.6789").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(0.0))),
            Token::lexer("0.00000000").next()
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(-1.23))),
            Token::lexer("-1.23").next()
        );
        assert_eq!(
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer("012345.6789e123").next()
        );
        assert_eq!(
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer("-012345.6789e123").next()
        );
        assert_eq!(
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer("1.").next()
        );
        assert_eq!(
            Some((Err(LexError::UnrecognizedToken), 0..15)),
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
        assert_eq!(
            Some(Err(LexError::UnrecognizedToken)),
            Token::lexer("1name").next()
        );
        assert_eq!(
            vec![
                Ok(Token::Name("dashed")),
                Err(LexError::UnrecognizedToken),
                Ok(Token::Name("name"))
            ],
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

    #[test]
    fn graphql_ruby_compatibility_test() {
        assert_eq!(
            Some(Ok(Token::StringValue(
                "This is a string with a newline \n Not allowed!".into()
            ))),
            Token::lexer_with_extras(
                "\"This is a string with a newline \n Not allowed!\"",
                Extras {
                    graphql_ruby_compatibility: true
                },
            )
            .next(),
        );
        assert_eq!(
            vec![Ok(Token::IntValue(123)), Ok(Token::Name("A"))],
            Token::lexer_with_extras(
                "123A",
                Extras {
                    graphql_ruby_compatibility: true
                },
            )
            .take(2)
            .collect::<Vec<_>>(),
        );
    }
}
