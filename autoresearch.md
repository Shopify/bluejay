# Autoresearch: bluejay-parser performance optimization

## Objective
Optimize the bluejay-parser GraphQL parser for speed. The primary workload is parsing the GitHub schema (~50k lines) and a kitchen sink executable document. The parser uses logos for lexing and a hand-written recursive descent parser for AST construction.

## Metrics
- **Primary**: total_µs (µs, lower is better) — sum of schema + kitchen_sink parse times
- **Secondary**: schema_µs, kitchen_sink_µs

## How to Run
`./autoresearch.sh` — outputs `METRIC name=number` lines.

## Files in Scope
- `bluejay-parser/src/lexer/logos_lexer.rs` — Main lexer (logos-based tokenizer)
- `bluejay-parser/src/lexer/logos_lexer/string_lexer.rs` — String value lexer
- `bluejay-parser/src/lexer/logos_lexer/block_string_lexer.rs` — Block string lexer
- `bluejay-parser/src/ast/tokens.rs` — Token stream with peek/lookahead (VecDeque buffer)
- `bluejay-parser/src/ast/parse.rs` — Parse trait and entry point
- `bluejay-parser/src/ast/from_tokens.rs` — FromTokens trait
- `bluejay-parser/src/ast/value.rs` — Value parsing (recursive)
- `bluejay-parser/src/ast/definition/` — All definition parsing (type defs, schema, etc.)
- `bluejay-parser/src/ast/executable/` — Executable document parsing
- `bluejay-parser/src/lexical_token.rs` — LexicalToken enum
- `bluejay-parser/src/lexical_token/*.rs` — Token types (Name, Punctuator, etc.)
- `bluejay-parser/src/span.rs` — Span type
- `bluejay-core/src/` — Core traits (can be changed if backwards compatible)
- `bluejay-parser/Cargo.toml` — Dependencies

## Off Limits
- Test files (unless there's a compelling reason)
- Other crates (bluejay-validator, bluejay-printer, etc.) unless necessary for compatibility
- Public API changes that break backwards compatibility

## Constraints
- All existing tests must pass (`cargo test -p bluejay-parser`)
- bluejay-core changes must be backwards compatible
- No large refactors initially — prefer targeted, measurable improvements

## Architecture Notes
- Lexer: logos generates a DFA-based lexer. Token enum maps logos tokens → LexicalToken.
- Token stream: `LexerTokens` wraps the lexer with a `VecDeque` buffer for lookahead (peek).
- Parser: recursive descent via `FromTokens` trait. `DefinitionDocument::parse_from_tokens` is the main entry point for schema parsing.
- The schema benchmark dominates (~2750µs vs ~10µs for kitchen sink).
- Schema has ~50k lines, heavy on type definitions with descriptions (block strings).

## What's Been Tried
- (baseline established)
