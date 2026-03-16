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

### Kept (improvements)
- **Rewrote block_string_lexer**: direct string processing instead of sub-lexer + Vec<Vec<Token>>. Eliminates ~9759 sub-lexer instantiations for the schema. (~10% schema improvement)
- **Optimized next_if_* methods**: single buffer peek+consume operation instead of separate peek-then-next. Avoids redundant VecDeque lookups.
- **Added Copy to DepthLimiter + preallocated Vec capacity** in DefinitionDocument.
- **Compact Span**: u32 start+len (8 bytes, Copy) instead of Range<usize> (16 bytes, Clone). Shrinks LexicalToken from ~48 to 32 bytes. **Big exec win: 48→45µs.**
- **Field alias: consume-then-check** instead of peek(1). Avoids buffering 2 tokens for the common no-alias case.
- **Lazy depth_limiter.bump()**: only bump when optional elements (args/directives/selection_set) actually exist. Avoids unnecessary Result creation for fields without sub-elements. **exec: 45→43µs.**
- **OperationType::is_match**: reverted hardcoded strings back to POSSIBLE_VALUES (no perf difference, keeps single source of truth).

### Discarded (regressions or no improvement)
- **Inline [Option<LexicalToken>; 2] buffer** replacing VecDeque — regression both times tried. VecDeque's ring buffer is well-optimized for this use case.
- **Vec::with_capacity(4) in SelectionSet/Arguments** — regression. Most selection sets are consumed by fields that don't have sub-selections, wasting allocation.
- **Name/Variable Copy** — no measurable improvement (Clone was already equivalent for these small types).
- **LTO in Cargo.toml** — build config, not parser code.

### Architecture insights
- VecDeque is surprisingly hard to beat for a 0-2 element lookahead buffer. The ring buffer's index arithmetic is cheaper than Option discriminant checks.
- Span size reduction has outsized impact because every token carries a Span, and smaller tokens mean better cache utilization in the VecDeque.
- The logos DFA is already very fast; most remaining time is in lexing + token construction.
- At ~28ns/token, we're approaching the theoretical minimum for a fully-featured GraphQL parser.
