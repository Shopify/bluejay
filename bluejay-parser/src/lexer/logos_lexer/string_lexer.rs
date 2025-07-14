use super::Token as OuterToken;
use crate::{
    lexer::{string_builder::CowStringBuilder, LexError, StringValueLexError},
    Span,
};
use logos::{Lexer, Logos};
use std::borrow::Cow;

#[derive(Logos, Debug)]
#[logos(subpattern hexdigit = r"[0-9A-Fa-f]")]
#[logos(subpattern fixedunicode = r"\\u[0-9A-Fa-f]{4}")]
pub(super) enum Token<'a> {
    #[regex(r#"[^\\"\n\r]+"#)]
    SourceCharacters(&'a str),

    #[regex(r"(?&fixedunicode)", parse_fixed_width_escaped_unicode)]
    FixedWidthEscapedUnicode(Option<char>),

    #[regex(
        r"(?&fixedunicode)(?&fixedunicode)",
        parse_surrogate_pair_escaped_unicode
    )]
    SurrogatePairEscapedUnicode(Result<(char, Option<char>), Span>),

    #[regex(r"\\u\{(?&hexdigit)+\}", parse_escaped_unicode)]
    EscapedUnicode(Option<char>),

    #[token("\\\"")]
    EscapedQuote,

    #[token("\\\\")]
    EscapedBackslash,

    #[token("\\/")]
    EscapedSlash,

    #[token("\\b")]
    EscapedBackspace,

    #[token("\\f")]
    EscapedFormFeed,

    #[token("\\n")]
    EscapedNewline,

    #[token("\\r")]
    EscapedCarriageReturn,

    #[token("\\t")]
    EscapedTab,

    #[token("\"")]
    Quote,

    #[token("\n")]
    Newline,

    #[token("\r")]
    CarriageReturn,
}

fn parse_escaped_unicode<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<char> {
    (lexer.slice().len() < 13).then_some(()).and_then(|_| {
        u32::from_str_radix(&lexer.slice()[3..lexer.slice().len() - 1], 16)
            .ok()
            .and_then(char::from_u32)
    })
}

fn parse_fixed_width_escaped_unicode<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<char> {
    u32::from_str_radix(&lexer.slice()[2..], 16)
        .ok()
        .and_then(char::from_u32)
}

fn parse_surrogate_pair_escaped_unicode<'a>(
    lexer: &mut Lexer<'a, Token<'a>>,
) -> Result<(char, Option<char>), Span> {
    let leading_value = u32::from_str_radix(&lexer.slice()[2..6], 16)
        .map_err(|_| Span::from(lexer.span().start..(lexer.span().start + 6)))?;
    let trailing_value = u32::from_str_radix(&lexer.slice()[8..], 16)
        .map_err(|_| Span::from((lexer.span().start + 6)..lexer.span().end))?;
    if (0xD800..=0xDBFF).contains(&leading_value) {
        if (0xDC00..=0xDFFF).contains(&trailing_value) {
            let raw_value = (leading_value - 0xD800) * 0x400 + (trailing_value - 0xDC00) + 0x10000;
            char::from_u32(raw_value)
                .map(|c| (c, None))
                .ok_or_else(|| lexer.span().into())
        } else {
            Err(Span::from((lexer.span().start + 6)..lexer.span().end))
        }
    } else {
        char::from_u32(leading_value)
            .ok_or_else(|| Span::from(lexer.span().start..(lexer.span().start + 6)))
            .and_then(|leading_char| {
                char::from_u32(trailing_value)
                    .ok_or_else(|| Span::from((lexer.span().start + 6)..lexer.span().end))
                    .map(|trailing_char| (leading_char, Some(trailing_char)))
            })
    }
}

impl<'a> Token<'a> {
    /// Returns a result indicating if the string was parsed successfully.
    /// Also bumps the outer lexer by the number of characters parsed.
    pub(super) fn parse(
        outer_lexer: &mut Lexer<'a, OuterToken<'a>>,
    ) -> Result<Cow<'a, str>, LexError> {
        let s = outer_lexer.remainder();
        let span_offset = outer_lexer.span().end;
        let lexer = Self::lexer(s);

        // starting Quote should already have been parsed

        let mut builder = CowStringBuilder::new(s.len());
        let mut errors = Vec::new();

        for (result, span) in lexer.spanned() {
            match result {
                Ok(token) => match token {
                    Self::SourceCharacters(s) => {
                        builder.append_source(s);
                    }
                    Self::EscapedUnicode(c) | Self::FixedWidthEscapedUnicode(c) => match c {
                        Some(c) => builder.append_char(c),
                        None => errors.push(StringValueLexError::InvalidUnicodeEscapeSequence(
                            Span::from(span) + span_offset,
                        )),
                    },
                    Self::SurrogatePairEscapedUnicode(chars) => match chars {
                        Ok((c, None)) => builder.append_char(c),
                        Ok((leading, Some(trailing))) => {
                            builder.append_char(leading);
                            builder.append_char(trailing);
                        }
                        Err(span) => errors.push(
                            StringValueLexError::InvalidUnicodeEscapeSequence(span + span_offset),
                        ),
                    },
                    Self::EscapedQuote => builder.append_char('\"'),
                    Self::EscapedBackslash => builder.append_char('\\'),
                    Self::EscapedSlash => builder.append_char('/'),
                    Self::EscapedBackspace => builder.append_char('\u{0008}'),
                    Self::EscapedFormFeed => builder.append_char('\u{000C}'),
                    Self::EscapedNewline => builder.append_char('\n'),
                    Self::EscapedCarriageReturn => builder.append_char('\r'),
                    Self::EscapedTab => builder.append_char('\t'),
                    Self::Quote => {
                        outer_lexer.bump(span.end);
                        return if errors.is_empty() {
                            Ok(builder.finish())
                        } else {
                            Err(LexError::StringValueInvalid(errors))
                        };
                    }
                    Self::Newline => {
                        if outer_lexer.extras.graphql_ruby_compatibility {
                            builder.append_char('\n');
                        } else {
                            errors.push(StringValueLexError::InvalidCharacters(
                                Span::from(span) + span_offset,
                            ));
                        }
                    }
                    Self::CarriageReturn => {
                        if outer_lexer.extras.graphql_ruby_compatibility {
                            builder.append_char('\r');
                        } else {
                            errors.push(StringValueLexError::InvalidCharacters(
                                Span::from(span) + span_offset,
                            ));
                        }
                    }
                },
                Err(()) => {
                    errors.push(StringValueLexError::InvalidCharacters(
                        Span::from(span) + span_offset,
                    ));
                }
            }
        }

        outer_lexer.bump(s.len());
        Err(LexError::UnrecognizedToken)
    }
}
