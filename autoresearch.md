# Autoresearch: bluejay-parser executable document performance

## Objective
Optimize the bluejay-parser GraphQL parser for parsing executable documents (queries/mutations) — the runtime hot path for API requests. The primary workload is `ExecutableDocument::parse` on a realistic ~912 line storefront API document with multiple queries, mutations, and fragments.

Schema parsing speed is a secondary concern (not runtime).

## Metrics
- **Primary**: exec_µs (µs, lower is better) — time to parse `data/large_executable.graphql` as `ExecutableDocument`
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
- `bluejay-parser/src/ast/executable/` — Executable document parsing (operation defs, selections, fragments)
- `bluejay-parser/src/ast/directives.rs` — Directive parsing
- `bluejay-parser/src/ast/arguments.rs` — Argument parsing
- `bluejay-parser/src/lexical_token.rs` — LexicalToken enum
- `bluejay-parser/src/lexical_token/*.rs` — Token types (Name, Punctuator, etc.)
- `bluejay-parser/src/span.rs` — Span type
- `bluejay-parser/src/ast/depth_limiter.rs` — Depth limiter
- `bluejay-core/src/` — Core traits (can change if backwards compatible)
- `bluejay-parser/Cargo.toml` — Dependencies
- `bluejay-parser/benches/parse.rs` — Benchmark file
- `data/large_executable.graphql` — Primary benchmark input (912 lines, realistic storefront queries)

## Off Limits
- Test files (unless compelling reason)
- Other crates (bluejay-validator, bluejay-printer, etc.) unless necessary
- Public API changes that break backwards compatibility

## Constraints
- All existing tests must pass (`cargo test -p bluejay-parser --features format-errors`)
- bluejay-core changes must be backwards compatible
- Focus on executable document parsing (runtime hot path)

## Architecture Notes
- Lexer: logos generates a DFA-based lexer. Token enum maps logos tokens → LexicalToken.
- Token stream: `LexerTokens` wraps the lexer with a `VecDeque` buffer for lookahead (peek 0 and 1 max).
- Parser: recursive descent via `FromTokens` trait. `ExecutableDocument::parse_from_tokens` is the entry for query parsing.
- Executable doc parsing: operations → selection sets → fields/fragments (recursive). Heavy on Name tokens, punctuators ({, }, (, ), :, ...), few strings.
- No block strings in typical executable documents (unlike schema definitions).

## What's Been Tried
- Rewrote block_string_lexer: direct string processing instead of sub-lexer + Vec<Vec<Token>> (~10% schema improvement)
- Optimized next_if_* methods: single buffer operation instead of peek-then-next
- Added Copy to DepthLimiter + preallocated Vec capacity in DefinitionDocument
- Tried replacing VecDeque with inline 2-element array buffer — regression, discarded
