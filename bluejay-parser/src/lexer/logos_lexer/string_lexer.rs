use crate::{
    lexer::{LexError, StringValueLexError},
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
    /// Returns a two-tuple
    /// - The first element is a result indicating if the string was parsed successfully.
    /// - The second element is how much the outer lexer should bump by.
    pub(super) fn parse(
        s: &'a <Self as Logos<'a>>::Source,
        span_offset: usize,
    ) -> (Result<Cow<'a, str>, LexError>, usize) {
        let lexer = Self::lexer(s);

        // starting Quote should already have been parsed

        let mut formatted = Cow::Borrowed("");
        let mut errors = Vec::new();

        for (result, span) in lexer.spanned() {
            match result {
                Ok(token) => match token {
                    Self::SourceCharacters(s) => {
                        formatted += s;
                    }
                    Self::EscapedUnicode(c) | Self::FixedWidthEscapedUnicode(c) => match c {
                        Some(c) => formatted.to_mut().push(c),
                        None => errors.push(StringValueLexError::InvalidUnicodeEscapeSequence(
                            Span::from(span) + span_offset,
                        )),
                    },
                    Self::SurrogatePairEscapedUnicode(chars) => match chars {
                        Ok((c, None)) => formatted.to_mut().push(c),
                        Ok((leading, Some(trailing))) => {
                            formatted.to_mut().push(leading);
                            formatted.to_mut().push(trailing);
                        }
                        Err(span) => errors.push(
                            StringValueLexError::InvalidUnicodeEscapeSequence(span + span_offset),
                        ),
                    },
                    Self::EscapedQuote => formatted.to_mut().push('\"'),
                    Self::EscapedBackslash => formatted.to_mut().push('\\'),
                    Self::EscapedSlash => formatted.to_mut().push('/'),
                    Self::EscapedBackspace => formatted.to_mut().push('\u{0008}'),
                    Self::EscapedFormFeed => formatted.to_mut().push('\u{000C}'),
                    Self::EscapedNewline => formatted.to_mut().push('\n'),
                    Self::EscapedCarriageReturn => formatted.to_mut().push('\r'),
                    Self::EscapedTab => formatted.to_mut().push('\t'),
                    Self::Quote => {
                        return (
                            if errors.is_empty() {
                                Ok(formatted)
                            } else {
                                Err(LexError::StringValueInvalid(errors))
                            },
                            span.end,
                        )
                    }
                },
                Err(()) => {
                    errors.push(StringValueLexError::InvalidCharacters(
                        Span::from(span) + span_offset,
                    ));
                }
            }
        }

        (Err(LexError::UnrecognizedToken), s.len())
    }
}
