use super::Token as OuterToken;
use crate::lexer::LexError;
use logos::{Lexer, Logos};
use std::borrow::Cow;
use std::cmp::min;

#[derive(Logos, Debug)]
pub(super) enum Token<'a> {
    #[regex(r#"[^"\\\n\r \t][^"\\\n\r]*"#)]
    #[token("\"")]
    #[token("\\")]
    BlockStringCharacters(&'a str),

    #[token("\n")]
    #[token("\r\n")]
    #[token("\r")]
    Newline,

    #[token(" ")]
    #[token("\t")]
    Whitespace(&'a str),

    #[token("\"\"\"")]
    BlockQuote,

    #[token("\\\"\"\"")]
    EscapedBlockQuote,
}

impl<'a> Token<'a> {
    /// Returns a result indicating if the string was parsed correctly.
    /// Also bumps the outer lexer by the number of characters parsed.
    pub(super) fn parse(
        outer_lexer: &mut Lexer<'a, OuterToken<'a>>,
    ) -> Result<Cow<'a, str>, LexError> {
        let mut lexer = Self::lexer(outer_lexer.remainder());

        // starting BlockQuote should already have been parsed

        let mut lines = vec![Vec::new()];

        while let Some(Ok(token)) = lexer.next() {
            match token {
                Self::BlockQuote => {
                    outer_lexer.bump(lexer.span().end);
                    return Ok(Self::block_string_value(lines));
                }
                Self::BlockStringCharacters(_) | Self::EscapedBlockQuote | Self::Whitespace(_) => {
                    lines.last_mut().unwrap().push(token)
                }
                Self::Newline => lines.push(Vec::new()),
            }
        }

        outer_lexer.bump(lexer.span().end);
        Err(LexError::UnrecognizedToken)
    }

    fn block_string_value(lines: Vec<Vec<Token<'a>>>) -> Cow<'a, str> {
        let common_indent = lines[1..]
            .iter()
            .filter_map(|line| {
                line.iter()
                    .position(|token| !matches!(token, Self::Whitespace(_)))
            })
            .min()
            .unwrap_or(0);

        let front_offset = lines.iter().enumerate().position(|(idx, line)| {
            let indent = if idx == 0 { 0 } else { common_indent };
            line[min(line.len(), indent)..]
                .iter()
                .any(|token| !matches!(token, Token::Whitespace(_)))
        });

        let end_offset = lines.iter().rev().position(|line| {
            line[min(line.len(), common_indent)..]
                .iter()
                .any(|token| !matches!(token, Token::Whitespace(_)))
        });

        let mut formatted = Cow::Borrowed("");

        if let Some((front_offset, end_offset)) = front_offset.zip(end_offset) {
            let start = front_offset;
            let end = lines.len() - end_offset;

            lines[start..end]
                .iter()
                .enumerate()
                .for_each(|(offset_idx, line)| {
                    let actual_idx = start + offset_idx;
                    let indent = if actual_idx == 0 { 0 } else { common_indent };
                    if offset_idx != 0 {
                        formatted += "\n";
                    }
                    line[min(line.len(), indent)..]
                        .iter()
                        .for_each(|token| match token {
                            Self::BlockStringCharacters(s) => {
                                formatted += *s;
                            }
                            Self::Whitespace(s) => {
                                formatted += *s;
                            }
                            Self::EscapedBlockQuote => {
                                formatted += "\"\"\"";
                            }
                            _ => {}
                        });
                });
        }

        formatted
    }
}
