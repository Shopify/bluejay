use logos::{Logos, Source};

#[derive(Logos, Debug)]
pub(super) enum Token<'a> {
    #[regex(r"[\u0009\u0020\u0021\u0023-\u005B\u005D-\uFFFF]+")]
    SourceCharacters(&'a str),

    #[regex(r"\\u[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]", |lex| char::from_u32(u32::from_str_radix(&lex.slice()[2..], 16).unwrap()).unwrap())]
    EscapedUnicode(char),

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

    #[error]
    Error,
}

impl<'a> Token<'a> {
    pub(super) fn parse(s: &'a <Self as Logos<'a>>::Source) -> String {
        let lexer = Self::lexer(s.slice(1..(s.len() - 1)).unwrap());

        let mut formatted = String::new();

        for token in lexer {
            match token {
                Self::SourceCharacters(s) => formatted.push_str(s),
                Self::EscapedUnicode(c) => formatted.push(c),
                Self::EscapedQuote => formatted.push('\u{0022}'),
                Self::EscapedBackslash => formatted.push('\u{005C}'),
                Self::EscapedSlash => formatted.push('\u{002F}'),
                Self::EscapedBackspace => formatted.push('\u{0008}'),
                Self::EscapedFormFeed => formatted.push('\u{000C}'),
                Self::EscapedNewline => formatted.push('\u{000A}'),
                Self::EscapedCarriageReturn => formatted.push('\u{000D}'),
                Self::EscapedTab => formatted.push('\u{0009}'),
                Self::Error => panic!("Unexpected error"),
            }
        }

        formatted
    }
}
