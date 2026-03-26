//! # GraphQL Operation Normalizer
//!
//! Produces a **canonical string representation** of a GraphQL operation, suitable for
//! generating stable signatures (hashes) that group operations by structure rather than
//! syntax. Two operations that differ only in whitespace, field ordering, alias names,
//! fragment style, argument values, or variable definitions will produce the same
//! signature.
//!
//! The normalizer eliminates the most common sources of trivial divergence while keeping
//! the implementation simple. It does **not** attempt full semantic equivalence — for
//! example, duplicate fields are not merged, and inline fragments are only merged at the
//! same nesting level, not across parent-child boundaries.
//!
//! ## Normalization Algorithm
//!
//! Given a parsed `ExecutableDocument` and an (optional) operation name:
//!
//! 1. **Resolve the operation** — find the target operation definition by name, or use the
//!    sole operation if unnamed. Error if ambiguous or missing.
//!
//! 2. **Build a normalized IR** from the operation's selection set, recursively processing
//!    each selection. This is a single bottom-up pass that builds and normalizes each level
//!    before returning it to the parent:
//!
//!    a. **Fields** — collect the field name (dropping any alias), sorted argument names,
//!       sorted directives, and recursively normalized child selections.
//!       ```graphql
//!       # input
//!       { myAlias: field(z: 1, a: 2) }
//!       # normalized
//!       query{field(a:,z:)}
//!       ```
//!
//!    b. **Fragment spreads** — expand inline: replace `...FragName` with
//!       `... on <TypeCondition> { <selections> }`, merging directives from both the spread
//!       and the fragment definition. Unused fragments are naturally excluded. Cycles are
//!       detected via a stack of currently-expanding fragment names.
//!       ```graphql
//!       # input
//!       { ...F }
//!       fragment F on User { name }
//!       # normalized — spread replaced with inline fragment, named fragment dropped
//!       query{...on User{name}}
//!       ```
//!
//!    c. **Inline fragments** — if bare (no type condition, no directives), flatten their
//!       children directly into the parent selection set. Otherwise, keep as-is.
//!       ```graphql
//!       # input — bare inline fragment (no type condition, no directives)
//!       { ... { name email } }
//!       # normalized — children flattened into parent
//!       query{email name}
//!       ```
//!
//!    d. **Merge inline fragments** — at each level, merge inline fragments that share the
//!       same `(type_condition, directives)` pair into a single inline fragment, combining
//!       their child selections.
//!       ```graphql
//!       # input — two inline fragments on same type
//!       { ... on User { name } ... on User { email } }
//!       # normalized — merged into one
//!       query{...on User{email name}}
//!       ```
//!
//!    e. **Sort selections** — fields first (alphabetically by field name), then inline
//!       fragments (by type condition, then by directives).
//!       ```graphql
//!       # input
//!       { z a ... on User { b } m }
//!       # normalized — fields sorted first, then inline fragments
//!       query{a m z ...on User{b}}
//!       ```
//!
//! 3. **Serialize** the normalized IR to a compact canonical string:
//!    - Operation type keyword (`query`, `mutation`, `subscription`) with no name.
//!    - Variable definitions are dropped entirely.
//!    - Argument and directive values are omitted (only `name:` is kept).
//!    - No whitespace except single spaces separating selections within `{ }`.
//!    - Argument names are sorted alphabetically within each argument list.
//!    - Directive names are sorted alphabetically.
//!    ```graphql
//!    # input
//!    query MyQuery($id: ID!) @z @a {
//!      user(id: $id) { name email }
//!    }
//!    # normalized — name dropped, vars dropped, directives sorted, values omitted
//!    query@a@z{user(id:){email name}}
//!    ```
//!
//! 4. **Signature** — optionally hash the canonical string with BLAKE3 to produce a
//!    stable hex digest.
//!
//! ## Module Structure
//!
//! - [`ir`] — Normalized IR types. (Step 2 data structures)
//! - [`build`] — Builds normalized IR from the parsed AST in a single recursive pass.
//!   (Steps 2a–2e)
//! - [`normalize`] — Entry point that orchestrates resolution, building, and serialization.
//!   (Steps 1–3)
//! - [`serialize`] — Writes the normalized IR to a canonical string. (Step 3)

mod build;
mod ir;
mod normalize;
mod serialize;

use bluejay_core::executable::ExecutableDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureError {
    OperationNotFound(String),
    AmbiguousOperation,
    NoOperations,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OperationNotFound(name) => write!(f, "operation not found: {name}"),
            Self::AmbiguousOperation => {
                write!(f, "multiple operations found; specify operation name")
            }
            Self::NoOperations => write!(f, "no operations in document"),
        }
    }
}

impl std::error::Error for SignatureError {}

pub fn normalize<E: ExecutableDocument>(
    doc: &E,
    op_name: Option<&str>,
) -> Result<String, SignatureError> {
    normalize::normalize_doc::<E>(doc, op_name)
}

pub fn signature<E: ExecutableDocument>(
    doc: &E,
    op_name: Option<&str>,
) -> Result<String, SignatureError> {
    let normalized = normalize::<E>(doc, op_name)?;
    Ok(blake3::hash(normalized.as_bytes()).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluejay_parser::ast::{executable::ExecutableDocument as ParserDoc, Parse};

    fn parse(input: &str) -> ParserDoc {
        ParserDoc::parse(input).result.expect("parse error")
    }

    // === Basic field sorting ===

    #[test]
    fn fields_sorted() {
        let doc = parse("{ z a m }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{a m z}");
    }

    #[test]
    fn nested_selection_sorting() {
        let doc = parse("{ parent { z a m } }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{parent{a m z}}");
    }

    #[test]
    fn duplicate_fields_preserved() {
        let doc = parse("{ a a a }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{a a a}");
    }

    // === Argument handling (values omitted) ===

    #[test]
    fn arguments_sorted_and_values_replaced() {
        let doc = parse("{ field(z: 1, a: 2, m: 3) }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field(a:,m:,z:)}");
    }

    #[test]
    fn all_value_types_replaced() {
        let doc = parse(
            r#"{ field(a: 42, b: 3.14, c: "hello", d: true, e: false, f: null, g: ENUM, h: [1,2], i: {x: 1}, j: $var) }"#,
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{field(a:,b:,c:,d:,e:,f:,g:,h:,i:,j:)}"
        );
    }

    // === Variable definitions and operation names dropped ===

    #[test]
    fn variable_definitions_dropped() {
        let doc = parse("query($z: String, $a: Int, $m: Boolean) { field }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field}");
    }

    #[test]
    fn operation_name_dropped() {
        let doc = parse("query MyQuery { field }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field}");
    }

    // === Aliases removed ===

    #[test]
    fn alias_removed() {
        let doc = parse("{ myAlias: someField }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{someField}");
    }

    #[test]
    fn aliased_fields_sorted_by_field_name() {
        // After alias removal, sort by the actual field name
        let doc = parse("{ z: fieldZ a: fieldA m: fieldM }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{fieldA fieldM fieldZ}"
        );
    }

    // === Fragment expansion ===

    #[test]
    fn fragment_expanded_to_inline() {
        let doc = parse(
            "query { ...F }
            fragment F on Query { a }",
        );
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on Query{a}}");
    }

    #[test]
    fn same_type_fragments_merged() {
        // Two fragments on same type get merged into one InlineFragment
        let doc = parse(
            "query { ...A ...B }
            fragment A on Query { a }
            fragment B on Query { b }",
        );
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on Query{a b}}");
    }

    #[test]
    fn different_type_fragments_not_merged() {
        let doc = parse(
            "query { ...A ...B }
            fragment A on TypeA { a }
            fragment B on TypeB { b }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on TypeA{a} ...on TypeB{b}}"
        );
    }

    #[test]
    fn transitive_fragments_expanded() {
        let doc = parse(
            "query { ...A }
            fragment A on T { ...B a }
            fragment B on T { ...C b }
            fragment C on T { c }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on T{a ...on T{b ...on T{c}}}}"
        );
    }

    #[test]
    fn unused_fragments_naturally_excluded() {
        let doc = parse(
            "query { ...Used }
            fragment Used on Query { a }
            fragment Unused on Query { b }",
        );
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on Query{a}}");
    }

    #[test]
    fn fragment_spread_directives_merged_with_def_directives() {
        let doc = parse(
            "query { ...F @skip(if: true) }
            fragment F on T @deprecated { a }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on T@deprecated@skip(if:){a}}"
        );
    }

    // === Inline fragment handling ===

    #[test]
    fn bare_inline_fragment_flattened() {
        // InlineFragment with no type condition and no directives → flattened
        let doc = parse("{ ... { field } }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field}");
    }

    #[test]
    fn inline_fragment_with_directive_preserved() {
        // Has directive → not flattened even without type condition
        let doc = parse("{ ... @include(if: true) { field } }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...@include(if:){field}}"
        );
    }

    #[test]
    fn inline_fragment_with_type_condition_preserved() {
        let doc = parse("{ ... on Query { field } }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on Query{field}}");
    }

    #[test]
    fn same_type_inline_fragments_merged() {
        let doc = parse("query { ... on Query { a } ... on Query { b } }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on Query{a b}}");
    }

    #[test]
    fn different_directive_inline_fragments_not_merged() {
        let doc = parse("query { ... on T @a { x } ... on T @b { y } }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on T@a{x} ...on T@b{y}}"
        );
    }

    // === Sort order: fields first, then inline fragments ===

    #[test]
    fn selection_sort_order() {
        let doc = parse(
            "query {
                ... on Query { inlined }
                ...Frag
                field
                regular
            }
            fragment Frag on Query { x }",
        );
        // Fields first (alpha), then InlineFragments (alpha by TC)
        // Frag expands to ... on Query { x }, merges with ... on Query { inlined }
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{field regular ...on Query{inlined x}}"
        );
    }

    #[test]
    fn field_without_directive_before_field_with_directive() {
        let doc = parse("query { name @uppercase name }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{name name@uppercase}");
    }

    #[test]
    fn fields_tiebreak_by_arguments() {
        let doc = parse("query { field(z: 1) field(a: 1) field }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{field field(a:) field(z:)}"
        );
    }

    #[test]
    fn fields_tiebreak_by_directives_after_arguments() {
        let doc = parse("query { field(a: 1) @z field(a: 1) @a }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{field(a:)@a field(a:)@z}"
        );
    }

    // === Directives ===

    #[test]
    fn directive_sorting() {
        let doc = parse("query @z @a @m { field }");
        assert_eq!(normalize(&doc, None).unwrap(), "query@a@m@z{field}");
    }

    #[test]
    fn directive_arguments_sorted() {
        let doc = parse("{ field @custom(z: 1, a: 2) }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field@custom(a:,z:)}");
    }

    #[test]
    fn repeated_directives_preserved() {
        let doc = parse(
            r#"query { products @filter(field: "price", op: "gt", value: "10") @filter(field: "category", op: "eq", value: "electronics") { id } }"#,
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{products@filter(field:,op:,value:)@filter(field:,op:,value:){id}}"
        );
    }

    // === Operation types ===

    #[test]
    fn mutation() {
        let doc = parse("mutation { doThing }");
        assert_eq!(normalize(&doc, None).unwrap(), "mutation{doThing}");
    }

    #[test]
    fn subscription() {
        let doc = parse("subscription { onEvent }");
        assert_eq!(normalize(&doc, None).unwrap(), "subscription{onEvent}");
    }

    // === Error cases ===

    #[test]
    fn error_operation_not_found() {
        let doc = parse("query Foo { a }");
        assert_eq!(
            normalize(&doc, Some("Bar")),
            Err(SignatureError::OperationNotFound("Bar".to_string()))
        );
    }

    #[test]
    fn error_ambiguous_operation() {
        let doc = parse("query A { a } query B { b }");
        assert_eq!(
            normalize(&doc, None),
            Err(SignatureError::AmbiguousOperation)
        );
    }

    #[test]
    fn error_no_operations() {
        let doc = parse("fragment F on Query { a }");
        assert_eq!(normalize(&doc, None), Err(SignatureError::NoOperations));
    }

    #[test]
    fn named_operation_selection() {
        let doc = parse("query A { a } query B { b }");
        assert_eq!(normalize(&doc, Some("B")).unwrap(), "query{b}");
    }

    // === Signature hash ===

    #[test]
    fn signature_hash() {
        let doc = parse("{ field }");
        let normalized = normalize(&doc, None).unwrap();
        assert_eq!(normalized, "query{field}");
        let sig = signature(&doc, None).unwrap();
        let expected = blake3::hash(b"query{field}").to_hex().to_string();
        assert_eq!(sig, expected);
    }

    // === Idempotency ===

    #[test]
    fn deterministic() {
        let input = "query @dir { b a field(x: 1) }";
        let doc = parse(input);
        assert_eq!(
            normalize(&doc, None).unwrap(),
            normalize(&doc, None).unwrap()
        );
    }

    // === Canonical form: different representations → same hash ===

    #[test]
    fn fragments_vs_inline_same_hash() {
        let with_fragment = parse(
            "query { ...F }
            fragment F on T { a b }",
        );
        let with_inline = parse("query { ... on T { a b } }");
        assert_eq!(
            normalize(&with_fragment, None).unwrap(),
            normalize(&with_inline, None).unwrap(),
        );
    }

    #[test]
    fn alias_vs_no_alias_same_hash() {
        let with_alias = parse("{ myAlias: field }");
        let without_alias = parse("{ field }");
        assert_eq!(
            normalize(&with_alias, None).unwrap(),
            normalize(&without_alias, None).unwrap(),
        );
    }

    #[test]
    fn different_values_same_hash() {
        let a = parse(r#"{ field(arg: "hello") }"#);
        let b = parse(r#"{ field(arg: "world") }"#);
        assert_eq!(normalize(&a, None).unwrap(), normalize(&b, None).unwrap(),);
    }

    #[test]
    fn different_variable_names_same_hash() {
        let a = parse("query($foo: String) { field(arg: $foo) }");
        let b = parse(r#"query($bar: String! = "Thing") { field(arg: $bar) }"#);
        assert_eq!(normalize(&a, None).unwrap(), normalize(&b, None).unwrap(),);
    }

    #[test]
    fn reordered_inline_fragments_same_hash() {
        let a = parse("query { ... on Query { a } ... on Query { b } }");
        let b = parse("query { ... on Query { b } ... on Query { a } }");
        assert_eq!(normalize(&a, None), normalize(&b, None));
    }

    // === Edge cases ===

    #[test]
    fn fragment_cycle_detected_and_skipped() {
        let doc = parse(
            "query { ...A }
            fragment A on T { field ...B }
            fragment B on T { other ...A }",
        );
        let result = normalize(&doc, None).unwrap();
        // Second ...A is a cycle — silently skipped
        assert_eq!(result, "query{...on T{field ...on T{other}}}");
    }

    #[test]
    fn bare_inline_fragment_flattened_then_coexists_with_typed() {
        let doc = parse("query { ... { a } ... on Query { b } }");
        // Bare fragment flattened to field `a`, not merged with typed fragment
        assert_eq!(normalize(&doc, None).unwrap(), "query{a ...on Query{b}}");
    }

    #[test]
    fn expanded_fragment_merges_with_existing_inline() {
        let doc = parse(
            "query { ... on T @a { x } ...F }
            fragment F on T @a { y }",
        );
        // F expands to ... on T @a { y }, merges with existing ... on T @a { x }
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on T@a{x y}}");
    }

    #[test]
    fn operation_directives_with_arguments() {
        let doc = parse("query @z(x: 1) @a(b: 2, c: 3) @m { field }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query@a(b:,c:)@m@z(x:){field}"
        );
    }

    #[test]
    fn inline_fragment_with_multiple_directives() {
        let doc = parse("query { ... on User @skip(if: true) @include(if: false) { name } }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on User@include(if:)@skip(if:){name}}"
        );
    }

    #[test]
    fn fragment_spread_multiple_directives_on_both_spread_and_def() {
        let doc = parse(
            "query { ...F @b @a }
            fragment F on T @d @c { field }",
        );
        // Spread directives [@b, @a] + def directives [@d, @c] → merged and sorted
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on T@a@b@c@d{field}}"
        );
    }

    #[test]
    fn three_same_type_inline_fragments_merged() {
        let doc = parse("query { ... on T { a } ... on T { b } ... on T { c } }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{...on T{a b c}}");
    }

    #[test]
    fn three_same_type_fragments_with_interleaved_other() {
        // IF_A, IF_B, IF_A2, IF_A3 — tests swap_remove doesn't skip elements
        let doc = parse(
            "query {
                ... on T { a }
                ... on Other { x }
                ... on T { b }
                ... on T { c }
            }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on Other{x} ...on T{a b c}}"
        );
    }

    #[test]
    fn inline_fragment_sort_multiple_type_conditions() {
        let doc = parse(
            "query {
                ... on Zebra { z }
                ... on Admin { a }
                ... on Middle { m }
            }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...on Admin{a} ...on Middle{m} ...on Zebra{z}}"
        );
    }
}
