//! Safety tests for the lexer with adversarial inputs.
//!
//! The lexer operates on untrusted inputs. These tests make sure that,
//! for hostile or malformed inputs, the lexer:
//! - does not panic
//! - always terminates and makes progress
//! - consumes the full input
//! - only returns spans that are in bounds and on `char` boundaries
//!
//! Spans that are out of bounds or not on `char` boundaries can cause
//! panics or out-of-bounds reads in downstream error formatting, which
//! slices the source by span.

use super::{Extras, Token};
use crate::lexer::{LexError, Lexer as _, LogosLexer, StringValueLexError};
use logos::Logos;

/// Lex the full input and assert the safety properties.
/// Return the number of items that the lexer produced.
fn assert_lexes_safely_with_extras(input: &str, extras: Extras) -> usize {
    let mut items = 0usize;
    let mut previous_end = 0usize;
    let mut lexer = Token::lexer_with_extras(input, extras);
    while let Some(result) = lexer.next() {
        let span = lexer.span();
        items += 1;
        // Each item must consume at least one byte, so the item count
        // bounded by the input length shows that the lexer makes progress.
        assert!(
            items <= input.len(),
            "the lexer must make progress on {input:?}"
        );
        assert!(
            span.start <= span.end && span.end <= input.len(),
            "span {span:?} is out of bounds on {input:?}"
        );
        assert!(
            input.is_char_boundary(span.start) && input.is_char_boundary(span.end),
            "span {span:?} is not on a char boundary on {input:?}"
        );
        assert!(
            previous_end <= span.start,
            "span {span:?} moves backwards on {input:?}"
        );
        previous_end = span.end;
        if let Err(LexError::StringValueInvalid(errors)) = result {
            for error in errors {
                let inner = match error {
                    StringValueLexError::InvalidUnicodeEscapeSequence(span)
                    | StringValueLexError::InvalidCharacters(span) => span.byte_range(),
                };
                assert!(
                    inner.start <= inner.end && inner.end <= input.len(),
                    "inner span {inner:?} is out of bounds on {input:?}"
                );
                assert!(
                    input.is_char_boundary(inner.start) && input.is_char_boundary(inner.end),
                    "inner span {inner:?} is not on a char boundary on {input:?}"
                );
            }
        }
    }
    // The lexer must consume the full input. A lexer that stops early
    // makes the parser accept a document prefix and silently ignore
    // the rest.
    assert_eq!(
        "",
        lexer.remainder(),
        "the lexer must consume the full input on {input:?}"
    );
    items
}

fn assert_lexes_safely(input: &str) -> usize {
    assert_lexes_safely_with_extras(input, Extras::default())
}

/// A collection of hostile and malformed inputs.
fn adversarial_corpus() -> Vec<String> {
    let mut corpus: Vec<String> = Vec::new();

    // Runs of quotes of each length, alone and followed by other tokens
    for n in 0..=13 {
        corpus.push("\"".repeat(n));
        corpus.push(format!("{} name", "\"".repeat(n)));
        corpus.push(format!("name {}", "\"".repeat(n)));
    }

    // Backslash torture
    corpus.push("\\".repeat(9));
    corpus.push(format!("\"{}\"", "\\".repeat(9)));
    corpus.push(format!("\"{}", "\\".repeat(9)));
    corpus.push("\"\\".into());
    corpus.push("\"\\\"".into());

    // Block strings with escaped closers, terminated and unterminated
    corpus.push(format!("\"\"\"{}", "\\\"\"\"".repeat(5)));
    corpus.push(format!("\"\"\"{}\"\"\"", "\\\"\"\"".repeat(5)));
    corpus.push("\"\"\"\\".into());
    corpus.push("\"\"\"\\\"".into());
    corpus.push("\"\"\"\\\"\"".into());
    corpus.push("\"\"\"a\"\"a\"\"\"".into());

    // Unicode escape sequences
    corpus.push(format!("\"\\u{{{}}}\"", "0".repeat(1_000)));
    corpus.push("\"\\u".into());
    corpus.push("\"\\u{".into());
    corpus.push("\"\\u{12".into());
    corpus.push("\"\\uD800".into());
    corpus.push("\"\\uD800\"".into());
    corpus.push("\"\\uD800\\u0041\"".into());
    corpus.push("\"\\uD83D\\uD83D\"".into());
    corpus.push("\"\\uDC00\\uD800\"".into());
    corpus.push("\"\\uFFFF\\uFFFF\"".into());
    corpus.push("\"\\u{110000}\\u{FFFFFFFF}\"".into());

    // Number torture
    corpus.push("-".into());
    corpus.push("--1".into());
    corpus.push("-.5".into());
    corpus.push("+1".into());
    corpus.push("1e".into());
    corpus.push("1e+".into());
    corpus.push("1e-".into());
    corpus.push("1.".into());
    corpus.push("1..2".into());
    corpus.push("1.2.3.4".into());
    corpus.push("00".into());
    corpus.push("01".into());
    corpus.push("-0".into());
    corpus.push("9".repeat(200));
    corpus.push(format!("-{0}.{0}e-{0}", "9".repeat(100)));
    corpus.push("123abc_.456".into());

    // Control characters, raw and inside strings
    let control_characters: String = (0u8..0x20).map(char::from).collect();
    corpus.push(control_characters.clone());
    corpus.push(format!("\"{control_characters}\""));
    corpus.push(format!("\"\"\"{control_characters}\"\"\""));
    corpus.push("\u{7F}".into());
    corpus.push("\0".into());
    corpus.push("#\0\u{7F}".into());

    // Multi-byte characters in tricky positions
    corpus.push("é".into());
    corpus.push("🔥".into());
    corpus.push("a\u{FEFF}b".into());
    corpus.push("1\u{FEFF}2".into());
    corpus.push("\"🔥".into());
    corpus.push("\"\"\"🔥".into());
    corpus.push("\"\"\"é\né\r🔥\r\n é\"\"\"".into());
    corpus.push("é123".into());
    corpus.push("123é".into());
    corpus.push("$é".into());
    corpus.push("#é".into());

    // Carriage returns without line feeds
    corpus.push("\"\"\"a\rb\r\"\"\"".into());
    corpus.push("\"a\rb\"".into());
    corpus.push("\r".into());

    // Comments without terminating newlines
    corpus.push("#".into());
    corpus.push(format!("#{}", "#".repeat(100)));

    // Punctuator fragments and dollar signs
    corpus.push("$".repeat(100));
    corpus.push(".".repeat(100));
    corpus.push("..".into());
    corpus.push("....".into());
    corpus.push(".....!".into());

    // Mixed garbage
    corpus.push("{\"\\%^&*\0é\"\"\"}".into());
    corpus.push("query { field(arg: \"unterminated }".into());

    corpus
}

/// A well-formed document with all token types, comments,
/// and multi-byte characters.
fn kitchen_sink() -> String {
    let mut source = String::from("\u{FEFF}");
    source.push_str(
        "query Kitchen($sink: [Int!] = -0) @dir(a: 1.5e-3, b: \"\\u0041\\u{1F525}\\uD83D\\uDD25\") {\n",
    );
    source.push_str(
        "  field(arg: \"\"\"\n    block é\n      indented\n  \\\"\"\"\n  \"\"\") # comment\r\n",
    );
    source.push_str("  ... on Thing { a, b }\r");
    source.push_str("  \"string with escapes \\n \\t \\\" \\\\ /\"\n");
    source.push_str("}\n");
    source
}

#[test]
fn adversarial_corpus_lexes_safely() {
    for input in adversarial_corpus() {
        assert_lexes_safely(&input);
        assert_lexes_safely_with_extras(
            &input,
            Extras {
                graphql_ruby_compatibility: true,
            },
        );
    }
}

#[test]
fn kitchen_sink_lexes_safely() {
    let source = kitchen_sink();
    let items = assert_lexes_safely(&source);
    assert!(items > 0);
}

/// Truncation at each char boundary simulates unexpected end of input
/// in each lexer state. Suffixes simulate torn or corrupted inputs.
#[test]
fn truncated_documents_lex_safely() {
    let source = kitchen_sink();
    for index in 0..=source.len() {
        if source.is_char_boundary(index) {
            assert_lexes_safely(&source[..index]);
            assert_lexes_safely(&source[index..]);
        }
    }
}

/// Large pathological inputs must complete in linear-like time and
/// without unbounded memory usage. A regression to super-linear
/// behavior makes this test very slow or makes it time out.
#[test]
fn large_pathological_inputs_terminate() {
    let large_inputs = [
        "\"".repeat(50_000),
        "\\".repeat(50_000),
        format!("\"{}\"", "a".repeat(100_000)),
        format!("\"{}\"", "\\n".repeat(50_000)),
        format!("\"{}", "\\u{1F5".repeat(20_000)),
        format!("\"\\u{{{}}}\"", "F".repeat(100_000)),
        format!("\"\"\"{}\"\"\"", "x é\n".repeat(25_000)),
        format!("\"\"\"{}", "\\\"\"\"".repeat(25_000)),
        format!("\"\"\"{}\"\"\"", " \n".repeat(50_000)),
        "$".repeat(50_000),
        "9 ".repeat(50_000),
        "0 ".repeat(50_000),
        ".".repeat(50_000),
        format!("#{}", "c".repeat(100_000)),
        // Large ASCII tokens and large ASCII ignored runs crashed
        // unoptimized builds with logos 0.15. Logos 0.16 lexes them
        // with bounded stack usage.
        "9".repeat(100_000),
        format!("-1.{0}e-{0}", "9".repeat(50_000)),
        " ".repeat(100_000),
        "\t \r\n,".repeat(20_000),
        // Keep runs of byte order marks below the crash threshold for
        // unoptimized builds. See large_multibyte_runs_terminate.
        "\u{FEFF}\t \r\n,".repeat(1_000),
    ];
    for input in large_inputs {
        assert_lexes_safely(&input);
    }
}

/// With logos 0.16, the generated matchers for runs of multi-byte
/// characters use stack space proportional to the run length in
/// unoptimized builds. One thousand multi-byte characters use
/// approximately 1 MiB of stack. Optimized builds compile the
/// recursion into loops, so release builds accept runs of all lengths.
/// The affected inputs are strings with many multi-byte characters and
/// runs of many byte order marks.
/// Logos 0.15 had the same problem for numeric tokens and for runs of
/// ASCII ignored characters. Logos 0.16 corrected those cases and
/// introduced the multi-byte problem for strings.
/// This test pins the current stack usage with some headroom.
/// If it starts to abort with a stack overflow after a logos upgrade,
/// then the stack usage per character became worse, which makes the
/// denial-of-service risk worse.
#[test]
fn long_token_stack_usage() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            assert_lexes_safely(&"9".repeat(1_000));
            assert_lexes_safely(&format!("-1.{}e-9", "9".repeat(1_000)));
            assert_lexes_safely(&" ".repeat(1_000));
            assert_lexes_safely(&"\u{FEFF}\t \r\n,".repeat(200));
            assert_lexes_safely(&format!("\"{}\"", "é".repeat(500)));
        })
        .unwrap()
        .join()
        .unwrap();
}

/// One large run of multi-byte characters must lex safely. With
/// logos 0.16, these inputs overflow the stack in unoptimized builds,
/// and the process aborts. Optimized builds are not affected.
/// See long_token_stack_usage for the details.
/// This test is ignored because a failure aborts the full test process.
/// Try to enable this test again after each logos upgrade:
/// run `cargo test -p bluejay-parser --lib -- --ignored` in a debug
/// build. If all tests pass, remove the ignore attribute.
#[test]
#[ignore = "logos 0.16 overflows the stack on long runs of multi-byte characters in unoptimized builds; try to re-enable after the next logos upgrade"]
fn large_multibyte_runs_terminate() {
    let large_inputs = [
        format!("\"{}\"", "é".repeat(50_000)),
        format!("\"{}\"", "🔥".repeat(25_000)),
        "\u{FEFF}".repeat(30_000),
        "\u{FEFF}\t \r\n,".repeat(20_000),
    ];
    for input in large_inputs {
        assert_lexes_safely(&input);
    }
}

/// The max tokens limit is a denial-of-service protection.
/// It must stop the lexer early for all inputs.
#[test]
fn max_tokens_bounds_adversarial_corpus() {
    for input in adversarial_corpus() {
        let mut lexer = LogosLexer::new(&input).with_max_tokens(Some(8));
        let items = (&mut lexer).count();
        // 8 tokens, plus possibly interleaved errors, plus the final
        // max tokens error
        assert!(
            items <= input.len() + 1,
            "the lexer must terminate on {input:?}"
        );
        assert!(
            lexer.token_count() <= 9,
            "the lexer must stop counting after the limit on {input:?}"
        );
    }
}
