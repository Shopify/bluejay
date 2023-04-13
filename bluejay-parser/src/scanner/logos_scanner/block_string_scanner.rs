use logos::Logos;
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
    pub(super) fn parse(s: &'a <Self as Logos<'a>>::Source) -> Result<(String, usize), ()> {
        let mut lexer = Self::lexer(s);

        // starting BlockQuote should already have been parsed

        let mut lines = vec![Vec::new()];

        while let Some(Ok(token)) = lexer.next() {
            match token {
                Self::BlockQuote => {
                    let consumed = s.len() - lexer.remainder().len();
                    return Ok((Self::block_string_value(lines), consumed));
                }
                Self::BlockStringCharacters(_) | Self::EscapedBlockQuote | Self::Whitespace(_) => {
                    lines.last_mut().unwrap().push(token)
                }
                Self::Newline => lines.push(Vec::new()),
            }
        }

        Err(())
    }

    fn block_string_value(lines: Vec<Vec<Token<'a>>>) -> String {
        let mut common_indent = None;

        for line in &lines[1..] {
            let leading_whitespace = line
                .iter()
                .position(|token| !matches!(token, Self::Whitespace(_)));
            if let Some(leading_whitespace) = leading_whitespace {
                if let Some(ci) = common_indent {
                    common_indent = Some(min(ci, leading_whitespace));
                } else {
                    common_indent = Some(leading_whitespace);
                }
            }
        }

        let common_indent = common_indent.unwrap_or(0);

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

        let mut formatted = String::new();

        if let Some(front_offset) = front_offset {
            if let Some(end_offset) = end_offset {
                let start = front_offset;
                let end = lines.len() - end_offset;
                formatted.extend(lines[start..end].iter().enumerate().flat_map(
                    |(offset_idx, line)| {
                        let actual_idx = start + offset_idx;
                        let indent = if actual_idx == 0 { 0 } else { common_indent };
                        let prepend = if offset_idx == 0 { "" } else { "\u{000A}" };
                        std::iter::once(prepend).chain(
                            line[min(line.len(), indent)..].iter().filter_map(
                                |token| match token {
                                    Self::BlockStringCharacters(s) => Some(*s),
                                    Self::Whitespace(s) => Some(*s),
                                    Self::EscapedBlockQuote => Some("\"\"\""),
                                    _ => None,
                                },
                            ),
                        )
                    },
                ))
            }
        }

        formatted
    }
}
