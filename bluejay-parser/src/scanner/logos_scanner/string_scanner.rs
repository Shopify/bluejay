use crate::Span;
use logos::{Lexer, Logos, Source};

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
    pub(super) fn parse(
        s: &'a <Self as Logos<'a>>::Source,
        span_offset: usize,
    ) -> Result<String, Vec<Span>> {
        let lexer = Self::lexer(s.slice(1..(s.len() - 1)).unwrap());

        let mut formatted = String::new();
        let mut errors = Vec::new();

        for (result, span) in lexer.spanned() {
            match result.expect("Unexpected error") {
                Self::SourceCharacters(s) => formatted.push_str(s),
                Self::EscapedUnicode(c) => match c {
                    Some(c) => formatted.push(c),
                    None => errors.push(Span::from(span) + span_offset + 1),
                },
                Self::FixedWidthEscapedUnicode(c) => match c {
                    Some(c) => formatted.push(c),
                    None => errors.push(Span::from(span) + span_offset + 1),
                },
                Self::SurrogatePairEscapedUnicode(chars) => match chars {
                    Ok((c, None)) => formatted.push(c),
                    Ok((leading, Some(trailing))) => {
                        formatted.push(leading);
                        formatted.push(trailing);
                    }
                    Err(span) => errors.push(span + span_offset + 1),
                },
                Self::EscapedQuote => formatted.push('\"'),
                Self::EscapedBackslash => formatted.push('\\'),
                Self::EscapedSlash => formatted.push('/'),
                Self::EscapedBackspace => formatted.push('\u{0008}'),
                Self::EscapedFormFeed => formatted.push('\u{000C}'),
                Self::EscapedNewline => formatted.push('\n'),
                Self::EscapedCarriageReturn => formatted.push('\r'),
                Self::EscapedTab => formatted.push('\t'),
            }
        }

        if errors.is_empty() {
            Ok(formatted)
        } else {
            Err(errors)
        }
    }
}
