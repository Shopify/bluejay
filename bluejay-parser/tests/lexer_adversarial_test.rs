//! Adversarial input tests through the public parse API.
//!
//! The parser operates on untrusted inputs. These tests make sure that
//! hostile or malformed documents do not cause panics, out-of-bounds
//! access, or unbounded work in the lexer, the parser, or the error
//! formatting that slices the source by span.

use bluejay_parser::{
    ast::{
        definition::{DefaultContext, DefinitionDocument},
        executable::ExecutableDocument,
        Parse, ParseOptions,
    },
    Error,
};

/// A collection of hostile and malformed documents.
fn adversarial_documents() -> Vec<String> {
    let mut documents: Vec<String> = Vec::new();

    // Runs of quotes of each length
    for n in 0..=13 {
        documents.push("\"".repeat(n));
        documents.push(format!("query {{ a(b: {}) }}", "\"".repeat(n)));
    }

    // Unterminated strings and block strings
    documents.push("query { field(arg: \"unterminated }".into());
    documents.push("query { field(arg: \"\"\"unterminated }".into());
    documents.push("\"\"\"unterminated description".into());
    documents.push("{ a(b: \"\\".into());
    documents.push(format!("{{ a(b: \"\"\"{}", "\\\"\"\"".repeat(5)));

    // Invalid escape sequences and lone surrogates
    documents.push("{ a(b: \"\\q\\u12\\u{}\\uD800\\uDC00\\uD800\") }".into());
    documents.push(format!("{{ a(b: \"\\u{{{}}}\") }}", "0".repeat(1_000)));

    // Number torture
    documents.push("{ a(b: 00, c: 1., d: 1e, e: 1.2.3, f: -, g: 123abc) }".into());
    documents.push(format!(
        "{{ a(b: {0}, c: -{0}.{0}e-{0}) }}",
        "9".repeat(100)
    ));

    // Control characters and multi-byte characters
    let control_characters: String = (0u8..0x20).map(char::from).collect();
    documents.push(control_characters.clone());
    documents.push(format!("{{ a(b: \"{control_characters}\") }}"));
    documents.push("query { fiéld🔥 }".into());
    documents.push("\u{FEFF}query\u{FEFF}{ a }\u{FEFF}".into());
    documents.push("{ a(b: \"🔥".into());

    // Punctuator fragments
    documents.push("{ ... }".into());
    documents.push("{ .. . .... }".into());
    documents.push("$".repeat(100));

    documents
}

/// Truncation at each char boundary simulates unexpected end of input.
fn truncations(source: &str) -> impl Iterator<Item = &str> {
    (0..=source.len())
        .filter(|&index| source.is_char_boundary(index))
        .map(|index| &source[..index])
}

fn assert_parses_safely(source: &str) {
    let executable = ExecutableDocument::parse(source);
    if let Err(errors) = executable.result {
        // Error conversion slices the source by span.
        // It must not panic for any input.
        let _ = Error::into_graphql_errors(source, errors);
    }
    let definition = DefinitionDocument::<DefaultContext>::parse(source);
    if let Err(errors) = definition.result {
        let _ = Error::into_graphql_errors(source, errors);
    }
}

#[test]
fn adversarial_documents_parse_safely() {
    for source in adversarial_documents() {
        assert_parses_safely(&source);
    }
}

#[test]
fn truncated_documents_parse_safely() {
    let source = "query Kitchen($sink: [Int!] = -0) @dir(a: 1.5e-3, b: \"\\u0041\\uD83D\\uDD25\") {\n  field(arg: \"\"\"\n    block é\n  \"\"\") # comment\r\n  ... on Thing { a, b }\r  \"string \\n \\t \\\" \\\\ /\"\n}\n";
    for truncated in truncations(source) {
        assert_parses_safely(truncated);
    }
}

/// Documents with one large numeric token or one large run of ASCII
/// ignored characters must parse safely. Logos 0.15 caused a stack
/// overflow for these inputs in unoptimized builds. Logos 0.16 parses
/// them with bounded stack usage.
#[test]
fn large_single_token_documents_parse_safely() {
    let sources = [
        format!("{{ a(b: {}) }}", "9".repeat(100_000)),
        format!("{{ a(b: -1.{0}e-{0}) }}", "9".repeat(50_000)),
        format!("query {}{{ a }}", " ".repeat(100_000)),
    ];
    for source in sources {
        assert_parses_safely(&source);
    }
}

/// Documents with one large run of multi-byte characters must parse
/// safely. With logos 0.16, these inputs overflow the stack in
/// unoptimized builds, and the process aborts. Optimized builds are
/// not affected. The max tokens limit does not protect against this,
/// because each input is a single token.
/// This test is ignored because a failure aborts the full test process.
/// Try to enable this test again after each logos upgrade:
/// run `cargo test -p bluejay-parser --test lexer_adversarial_test -- --ignored`
/// in a debug build. If all tests pass, remove the ignore attribute.
#[test]
#[ignore = "logos 0.16 overflows the stack on long runs of multi-byte characters in unoptimized builds; try to re-enable after the next logos upgrade"]
fn large_multibyte_documents_parse_safely() {
    let sources = [
        format!("{{ a(b: \"{}\") }}", "é".repeat(50_000)),
        format!("{}{{ a }}", "\u{FEFF}".repeat(30_000)),
    ];
    for source in sources {
        assert_parses_safely(&source);
    }
}

/// The max tokens limit is a denial-of-service protection.
/// It must bound the work for large hostile documents.
#[test]
fn max_tokens_bounds_large_documents() {
    let source = "{ a } ".repeat(10_000);
    let result = ExecutableDocument::parse_with_options(
        source.as_str(),
        ParseOptions {
            max_tokens: Some(100),
            ..Default::default()
        },
    );
    assert!(result.result.is_err());
    assert_eq!(101, result.token_count);
}
