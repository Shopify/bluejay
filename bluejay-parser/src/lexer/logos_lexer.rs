use crate::lexer::{LexError, Lexer};
use crate::lexical_token::{
    FloatValue, IntValue, LexicalToken, Name, Punctuator, PunctuatorType, StringValue, Variable,
};
use crate::Span;
use logos::Logos;
use std::borrow::Cow;

mod block_string_lexer;
#[cfg(test)]
mod safety_tests;
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
#[logos(skip(r"#[^\n\r]*", allow_greedy = true))] // comments
#[logos(extras = Extras)]
pub(crate) enum Token<'a> {
    // Punctuators
    #[token("!")]
    Bang,
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

    // VariableName
    #[regex(r"\$[_a-zA-Z][_0-9a-zA-Z]*", |lex| &lex.slice()[1..])]
    VariableName(&'a str),

    // Name
    #[regex(r"[_a-zA-Z][_0-9a-zA-Z]*")]
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

#[inline]
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
        .unwrap_or_else(|| lexer.remainder().len());

    lexer.bump(invalid_trail_bytes);

    if invalid_trail_bytes == 0 {
        Ok(())
    } else {
        Err(LexError::UnrecognizedToken)
    }
}

#[inline]
fn parse_integer<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> Result<i32, LexError> {
    validate_number_no_trailing_name_start(lexer).and_then(|_| {
        lexer
            .slice()
            .parse()
            .map_err(|_| LexError::IntegerValueTooLarge)
    })
}

#[inline]
fn parse_float<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> Result<f64, LexError> {
    validate_number_no_trailing_name_start(lexer).and_then(|_| {
        lexer
            .slice()
            .parse()
            .map_err(|_| LexError::FloatValueTooLarge)
    })
}

pub struct LogosLexer<'a> {
    inner: logos::Lexer<'a, Token<'a>>,
    token_count: usize,
    max_tokens: Option<usize>,
    exceeded_max_tokens: bool,
}

impl<'a> Iterator for LogosLexer<'a> {
    type Item = Result<LexicalToken<'a>, (LexError, Span)>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.exceeded_max_tokens {
            return None;
        }

        match self.inner.next() {
            Some(Ok(token)) => {
                self.token_count += 1;
                let span = Span::new(self.inner.span());

                if let Some(max) = self.max_tokens {
                    if self.token_count > max {
                        self.exceeded_max_tokens = true;
                        return Some(Err((LexError::MaxTokensExceeded { limit: max }, span)));
                    }
                }

                let lexical_token = match token {
                    Token::Bang => punctuator(PunctuatorType::Bang, span),
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
                    Token::VariableName(s) => LexicalToken::VariableName(Variable::new(s, span)),
                    Token::Name(s) => LexicalToken::Name(Name::new(s, span)),
                    Token::IntValue(val) => LexicalToken::IntValue(IntValue::new(val, span)),
                    Token::FloatValue(val) => LexicalToken::FloatValue(FloatValue::new(val, span)),
                    Token::StringValue(val) => {
                        LexicalToken::StringValue(StringValue::new(val, span))
                    }
                    Token::BlockStringValue(val) => {
                        LexicalToken::StringValue(StringValue::new(val, span))
                    }
                };
                Some(Ok(lexical_token))
            }
            Some(Err(err)) => Some(Err((err, Span::new(self.inner.span())))),
            None => None,
        }
    }
}

#[inline]
fn punctuator<'a>(pt: PunctuatorType, span: Span) -> LexicalToken<'a> {
    LexicalToken::Punctuator(Punctuator::new(pt, span))
}

impl<'a> Lexer<'a> for LogosLexer<'a> {
    fn empty_span(&self) -> Span {
        let n = self.inner.span().start;
        Span::new(n..n)
    }

    fn token_count(&self) -> usize {
        self.token_count
    }
}

impl<'a> LogosLexer<'a> {
    pub fn new(s: &'a <Token<'a> as Logos<'a>>::Source) -> Self {
        Self {
            inner: Token::lexer(s),
            token_count: 0,
            max_tokens: None,
            exceeded_max_tokens: false,
        }
    }

    pub fn with_graphql_ruby_compatibility(mut self, enabled: bool) -> Self {
        self.inner.extras.graphql_ruby_compatibility = enabled;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<usize>) -> Self {
        self.max_tokens = max_tokens;
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
    fn punctuator_test() {
        assert_eq!(
            vec![
                (Ok(Token::Bang), 0..1),
                (Ok(Token::Ampersand), 1..2),
                (Ok(Token::OpenRoundBracket), 2..3),
                (Ok(Token::CloseRoundBracket), 3..4),
                (Ok(Token::Ellipse), 4..7),
                (Ok(Token::Colon), 7..8),
                (Ok(Token::Equals), 8..9),
                (Ok(Token::At), 9..10),
                (Ok(Token::OpenSquareBracket), 10..11),
                (Ok(Token::CloseSquareBracket), 11..12),
                (Ok(Token::OpenBrace), 12..13),
                (Ok(Token::CloseBrace), 13..14),
                (Ok(Token::Pipe), 14..15),
            ],
            Token::lexer("!&()...:=@[]{}|")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // One dot and two dots are not valid tokens
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..1)],
            Token::lexer(".").spanned().collect::<Vec<_>>(),
        );
        // Two dots give one error that spans the failed match attempt
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..2)],
            Token::lexer("..").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![
                (Ok(Token::Ellipse), 0..3),
                (Err(LexError::UnrecognizedToken), 3..4),
            ],
            Token::lexer("....").spanned().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn variable_name_test() {
        assert_eq!(
            vec![(Ok(Token::VariableName("foo")), 0..4)],
            Token::lexer("$foo").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            Some(Ok(Token::VariableName("_v1"))),
            Token::lexer("$_v1").next(),
        );
        assert_eq!(
            Some(Ok(Token::VariableName("__typename"))),
            Token::lexer("$__typename").next(),
        );
        // A variable name must not start with a digit
        assert_eq!(
            vec![
                (Err(LexError::UnrecognizedToken), 0..1),
                (Err(LexError::UnrecognizedToken), 1..5),
            ],
            Token::lexer("$1foo").spanned().collect::<Vec<_>>(),
        );
        // A dollar sign alone is not a valid token
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..1)],
            Token::lexer("$").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![
                (Err(LexError::UnrecognizedToken), 0..1),
                (Ok(Token::Name("name")), 2..6),
            ],
            Token::lexer("$ name").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![
                (Ok(Token::VariableName("foo")), 0..4),
                (Ok(Token::Colon), 4..5),
            ],
            Token::lexer("$foo:").spanned().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn ignored_tokens_test() {
        // A byte order mark is ignored
        assert_eq!(
            vec![(Ok(Token::Name("query")), 3..8)],
            Token::lexer("\u{FEFF}query").spanned().collect::<Vec<_>>(),
        );
        // Commas and all white space characters are ignored
        assert_eq!(
            vec![
                (Ok(Token::Name("a")), 0..1),
                (Ok(Token::Name("b")), 2..3),
                (Ok(Token::Name("c")), 8..9),
            ],
            Token::lexer("a,b\t,,\r\nc").spanned().collect::<Vec<_>>(),
        );
        // Input with only ignored tokens gives no tokens
        assert_eq!(None, Token::lexer("\u{FEFF} \t\r\n,,").next());
        // A comment ends at a newline
        assert_eq!(
            vec![(Ok(Token::Name("x")), 3..4)],
            Token::lexer("#c\nx").spanned().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn string_edge_cases_test() {
        // The empty string
        assert_eq!(
            vec![(Ok(Token::StringValue("".into())), 0..2)],
            Token::lexer(r#""""#).spanned().collect::<Vec<_>>(),
        );
        // Two adjacent strings
        assert_eq!(
            vec![
                (Ok(Token::StringValue("".into())), 0..2),
                (Ok(Token::StringValue("a".into())), 3..6),
            ],
            Token::lexer(r#""" "a""#).spanned().collect::<Vec<_>>(),
        );
        // All simple escape sequences
        assert_eq!(
            vec![(
                Ok(Token::StringValue("\u{8}\u{c}\n\r\t/\\\"".into())),
                0..18,
            )],
            Token::lexer(r#""\b\f\n\r\t\/\\\"""#)
                .spanned()
                .collect::<Vec<_>>(),
        );
        // An invalid escape sequence
        assert_eq!(
            vec![(
                Err(LexError::StringValueInvalid(vec![
                    StringValueLexError::InvalidCharacters(Span::from(1..2)),
                ])),
                0..4,
            )],
            Token::lexer(r#""\q""#).spanned().collect::<Vec<_>>(),
        );
        // A unicode escape sequence with too few digits.
        // The inner error covers the full failed escape sequence.
        assert_eq!(
            vec![(
                Err(LexError::StringValueInvalid(vec![
                    StringValueLexError::InvalidCharacters(Span::from(1..5)),
                ])),
                0..6,
            )],
            Token::lexer(r#""\u12""#).spanned().collect::<Vec<_>>(),
        );
        // A unicode escape sequence with no digits.
        // The inner error covers the full failed escape sequence.
        assert_eq!(
            vec![(
                Err(LexError::StringValueInvalid(vec![
                    StringValueLexError::InvalidCharacters(Span::from(1..4)),
                ])),
                0..6,
            )],
            Token::lexer(r#""\u{}""#).spanned().collect::<Vec<_>>(),
        );
        // Unterminated strings consume the full remainder
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..4)],
            Token::lexer("\"abc").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..5)],
            Token::lexer("\"abc\\").spanned().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn block_string_edge_cases_test() {
        // Indentation is removed relative to the common indent
        assert_eq!(
            vec![(
                Ok(Token::BlockStringValue("first\n  second\nthird".into())),
                0..40,
            )],
            Token::lexer("\"\"\"\n    first\n      second\n    third\n\"\"\"")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // The first line keeps its indentation
        assert_eq!(
            vec![(Ok(Token::BlockStringValue("abc\ndef".into())), 0..15)],
            Token::lexer("\"\"\"abc\n  def\"\"\"")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // A block string with only white space is empty
        assert_eq!(
            vec![(Ok(Token::BlockStringValue("".into())), 0..13)],
            Token::lexer("\"\"\"   \n   \"\"\"")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // Blank leading and trailing lines are removed
        assert_eq!(
            vec![(Ok(Token::BlockStringValue("abc".into())), 0..16)],
            Token::lexer("\"\"\"\nabc\n\n   \n\"\"\"")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // An escaped triple quote at the start of the contents
        assert_eq!(
            vec![(Ok(Token::BlockStringValue("\"\"\"abc".into())), 1..14)],
            Token::lexer(r#" """\"""abc""" "#)
                .spanned()
                .collect::<Vec<_>>(),
        );
        // Multi-byte characters at the start of lines
        assert_eq!(
            vec![(Ok(Token::BlockStringValue("é\n é\né".into())), 0..15)],
            Token::lexer("\"\"\"é\n é\ré\"\"\"")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // Runs of quotes: four and five quotes are unterminated block strings
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..4)],
            Token::lexer("\"\"\"\"").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..5)],
            Token::lexer("\"\"\"\"\"").spanned().collect::<Vec<_>>(),
        );
        // Six quotes are an empty block string
        assert_eq!(
            vec![(Ok(Token::BlockStringValue("".into())), 0..6)],
            Token::lexer("\"\"\"\"\"\"").spanned().collect::<Vec<_>>(),
        );
        // Seven quotes are an empty block string and an unterminated string
        assert_eq!(
            vec![
                (Ok(Token::BlockStringValue("".into())), 0..6),
                (Err(LexError::UnrecognizedToken), 6..7),
            ],
            Token::lexer("\"\"\"\"\"\"\"").spanned().collect::<Vec<_>>(),
        );
        // An escaped triple quote directly before the end of the input
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..7)],
            Token::lexer("\"\"\"\\\"\"\"").spanned().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn token_spans_after_string_test() {
        // String parsing extends the outer lexer manually.
        // These tests make sure that subsequent spans stay correct.
        assert_eq!(
            vec![
                (Ok(Token::StringValue("abc".into())), 0..5),
                (Ok(Token::Name("name")), 6..10),
            ],
            Token::lexer("\"abc\" name").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![
                (Ok(Token::BlockStringValue("abc".into())), 0..9),
                (Ok(Token::Name("name")), 10..14),
            ],
            Token::lexer("\"\"\"abc\"\"\" name")
                .spanned()
                .collect::<Vec<_>>(),
        );
        // Errors in strings also consume the correct number of bytes
        assert_eq!(
            vec![
                (
                    Err(LexError::StringValueInvalid(vec![
                        StringValueLexError::InvalidCharacters(Span::from(1..2)),
                    ])),
                    0..4,
                ),
                (Ok(Token::Name("name")), 5..9),
            ],
            Token::lexer("\"\\q\" name").spanned().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn number_edge_cases_test() {
        // Boundary values for 32-bit signed integers
        assert_eq!(
            Some(Ok(Token::IntValue(i32::MAX))),
            Token::lexer("2147483647").next(),
        );
        assert_eq!(
            Some(Ok(Token::IntValue(i32::MIN))),
            Token::lexer("-2147483648").next(),
        );
        // A minus sign alone is not a valid token
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..1)],
            Token::lexer("-").spanned().collect::<Vec<_>>(),
        );
        // An exponent must have digits
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..2)],
            Token::lexer("1e").spanned().collect::<Vec<_>>(),
        );
        // A number must have at most one decimal point
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..5)],
            Token::lexer("1.2.3").spanned().collect::<Vec<_>>(),
        );
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..7)],
            Token::lexer("1.2e3.4").spanned().collect::<Vec<_>>(),
        );
        // Hexadecimal notation is not valid
        assert_eq!(
            vec![(Err(LexError::UnrecognizedToken), 0..4)],
            Token::lexer("0x10").spanned().collect::<Vec<_>>(),
        );
        // A comma terminates a number
        assert_eq!(
            vec![
                (Ok(Token::IntValue(1)), 0..1),
                (Ok(Token::IntValue(2)), 2..3),
            ],
            Token::lexer("1,2").spanned().collect::<Vec<_>>(),
        );
        // A punctuator terminates a number
        assert_eq!(
            vec![
                (Ok(Token::IntValue(123)), 0..3),
                (Ok(Token::CloseRoundBracket), 3..4),
            ],
            Token::lexer("123)").spanned().collect::<Vec<_>>(),
        );
        // Exponent variants
        assert_eq!(Some(Ok(Token::FloatValue(1e5))), Token::lexer("1E5").next(),);
        assert_eq!(
            Some(Ok(Token::FloatValue(1e5))),
            Token::lexer("1e+5").next(),
        );
        assert_eq!(
            Some(Ok(Token::FloatValue(1e-5))),
            Token::lexer("1e-5").next(),
        );
    }

    #[test]
    fn kitchen_sink_token_stream_test() {
        // Lex a document with all token types and all ignored token types,
        // and compare the full token stream, with spans, to the expected stream.
        let separators = [
            " ",
            ",",
            "\n",
            "\t",
            "\r\n",
            " # comment\n",
            " # comment\r\n",
            " #comment\r",
            "\u{FEFF}",
        ];
        let parts = vec![
            ("query", Token::Name("query")),
            ("MyQuery", Token::Name("MyQuery")),
            ("(", Token::OpenRoundBracket),
            ("$var", Token::VariableName("var")),
            (":", Token::Colon),
            ("[", Token::OpenSquareBracket),
            ("Int", Token::Name("Int")),
            ("!", Token::Bang),
            ("]", Token::CloseSquareBracket),
            ("=", Token::Equals),
            ("-42", Token::IntValue(-42)),
            (")", Token::CloseRoundBracket),
            ("@", Token::At),
            ("dir", Token::Name("dir")),
            ("{", Token::OpenBrace),
            ("...", Token::Ellipse),
            ("on", Token::Name("on")),
            ("&", Token::Ampersand),
            ("|", Token::Pipe),
            ("1.5e-3", Token::FloatValue(1.5e-3)),
            ("0.25", Token::FloatValue(0.25)),
            ("7e2", Token::FloatValue(7e2)),
            ("\"str \\u0041\"", Token::StringValue("str A".into())),
            ("\"\"\"block\"\"\"", Token::BlockStringValue("block".into())),
            ("}", Token::CloseBrace),
        ];
        let mut input = String::new();
        let mut expected = Vec::new();
        for (index, (text, token)) in parts.into_iter().enumerate() {
            input.push_str(separators[index % separators.len()]);
            let start = input.len();
            input.push_str(text);
            expected.push((Ok(token), start..input.len()));
        }
        input.push_str(" # trailing comment");
        assert_eq!(expected, Token::lexer(&input).spanned().collect::<Vec<_>>());
    }

    #[test]
    fn logos_lexer_iterator_test() {
        use crate::lexer::{Lexer, LogosLexer};
        use crate::lexical_token::{
            FloatValue, IntValue, LexicalToken, Name, Punctuator, PunctuatorType, StringValue,
            Variable,
        };
        use crate::Span;

        let input = "query $v 42 -3.5 \"s\" \"\"\"b\"\"\" @ {";
        let mut lexer = LogosLexer::new(input);
        let tokens: Vec<LexicalToken> = (&mut lexer).map(Result::unwrap).collect();
        assert_eq!(
            vec![
                LexicalToken::Name(Name::new("query", Span::new(0..5))),
                LexicalToken::VariableName(Variable::new("v", Span::new(6..8))),
                LexicalToken::IntValue(IntValue::new(42, Span::new(9..11))),
                LexicalToken::FloatValue(FloatValue::new(-3.5, Span::new(12..16))),
                LexicalToken::StringValue(StringValue::new("s".into(), Span::new(17..20))),
                LexicalToken::StringValue(StringValue::new("b".into(), Span::new(21..28))),
                LexicalToken::Punctuator(Punctuator::new(PunctuatorType::At, Span::new(29..30))),
                LexicalToken::Punctuator(Punctuator::new(
                    PunctuatorType::OpenBrace,
                    Span::new(31..32),
                )),
            ],
            tokens,
        );
        assert_eq!(8, lexer.token_count());
        assert_eq!(Span::new(32..32), lexer.empty_span());
    }

    #[test]
    fn logos_lexer_max_tokens_test() {
        use crate::lexer::LogosLexer;

        let input = "a b c d";
        let results: Vec<_> = LogosLexer::new(input)
            .with_max_tokens(Some(2))
            .map(|result| result.map_err(|(error, span)| (error, span.byte_range())))
            .collect();
        assert_eq!(3, results.len());
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert_eq!(
            Err((LexError::MaxTokensExceeded { limit: 2 }, 4..5)),
            results[2],
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
            Some(Ok(Token::StringValue(
                "This is a string with a carriage return \r Not allowed!".into()
            ))),
            Token::lexer_with_extras(
                "\"This is a string with a carriage return \r Not allowed!\"",
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
