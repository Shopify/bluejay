mod fragments;
mod normalize;
mod sort;

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

    #[test]
    fn fields_sorted() {
        let doc = parse("{ z a m }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{a m z}");
    }

    #[test]
    fn arguments_sorted() {
        let doc = parse("{ field(z: 1, a: 2, m: 3) }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field(a:0,m:0,z:0)}");
    }

    #[test]
    fn variables_sorted() {
        let doc = parse("query Foo($z: String, $a: Int, $m: Boolean) { field }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query Foo($a:Int,$m:Boolean,$z:String){field}"
        );
    }

    #[test]
    fn string_value_normalized() {
        let doc = parse(r#"{ field(arg: "hello world") }"#);
        assert_eq!(normalize(&doc, None).unwrap(), r#"query{field(arg:"")}"#);
    }

    #[test]
    fn numeric_values_normalized() {
        let doc = parse("{ field(a: 42, b: 3.14) }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field(a:0,b:0)}");
    }

    #[test]
    fn object_value_normalized() {
        let doc = parse(r#"{ field(arg: { z: "hello", a: 42, m: true }) }"#);
        assert_eq!(
            normalize(&doc, None).unwrap(),
            r#"query{field(arg:{a:0,m:true,z:""})}"#
        );
    }

    #[test]
    fn list_value_normalized() {
        let doc = parse("{ field(arg: [1, 2, 3]) }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field(arg:[])}");
    }

    #[test]
    fn preserved_values() {
        let doc = parse("{ field(a: true, b: false, c: null, d: SOME_ENUM) }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{field(a:true,b:false,c:null,d:SOME_ENUM)}"
        );
    }

    #[test]
    fn variable_reference_preserved() {
        let doc = parse("query($x: String) { field(arg: $x) }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query($x:String){field(arg:$x)}"
        );
    }

    #[test]
    fn fragment_ordering() {
        let doc = parse(
            "query { ...Z ...A }
            fragment Z on Query { z }
            fragment A on Query { a }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "fragment A on Query{a}fragment Z on Query{z}query{...A ...Z}"
        );
    }

    #[test]
    fn unused_fragments_excluded() {
        let doc = parse(
            "query { ...Used }
            fragment Used on Query { a }
            fragment Unused on Query { b }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "fragment Used on Query{a}query{...Used}"
        );
    }

    #[test]
    fn selection_sort_order() {
        let doc = parse(
            "query {
                ... on Query { inlined }
                ...Frag
                aliased: field
                regular
            }
            fragment Frag on Query { x }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "fragment Frag on Query{x}query{aliased:field regular ...Frag ... on Query{inlined}}"
        );
    }

    #[test]
    fn directive_sorting() {
        let doc = parse("query @z @a @m { field }");
        assert_eq!(normalize(&doc, None).unwrap(), "query@a@m@z{field}");
    }

    #[test]
    fn alias_format() {
        let doc = parse("{ myAlias: someField }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{myAlias:someField}");
    }

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
    fn named_operation_selection() {
        let doc = parse("query A { a } query B { b }");
        assert_eq!(normalize(&doc, Some("B")).unwrap(), "query B{b}");
    }

    #[test]
    fn idempotency() {
        let input = "query Foo($a: Int, $b: String) @dir { b a field(x: 1) }";
        let doc1 = parse(input);
        let normalized1 = normalize(&doc1, None).unwrap();
        let doc2 = parse(&normalized1);
        let normalized2 = normalize(&doc2, None).unwrap();
        assert_eq!(normalized1, normalized2);
    }

    #[test]
    fn signature_hash() {
        let doc = parse("{ field }");
        let normalized = normalize(&doc, None).unwrap();
        assert_eq!(normalized, "query{field}");
        let sig = signature(&doc, None).unwrap();
        let expected = blake3::hash(b"query{field}").to_hex().to_string();
        assert_eq!(sig, expected);
    }

    #[test]
    fn nested_selection_sorting() {
        let doc = parse("{ parent { z a m } }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{parent{a m z}}");
    }

    #[test]
    fn default_value_normalization() {
        let doc = parse(r#"query($x: String = "hello", $y: Int = 42) { field }"#);
        assert_eq!(
            normalize(&doc, None).unwrap(),
            r#"query($x:String="",$y:Int=0){field}"#
        );
    }

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

    #[test]
    fn transitive_fragments() {
        let doc = parse(
            "query { ...A }
            fragment A on Query { ...B a }
            fragment B on Query { ...C b }
            fragment C on Query { c }
            fragment Unused on Query { unused }",
        );
        let result = normalize(&doc, None).unwrap();
        assert!(result.contains("fragment A on Query"));
        assert!(result.contains("fragment B on Query"));
        assert!(result.contains("fragment C on Query"));
        assert!(!result.contains("Unused"));
    }

    #[test]
    fn inline_fragment_no_type_condition() {
        let doc = parse("{ ... @include(if: true) { field } }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{...@include(if:true){field}}"
        );
    }

    #[test]
    fn complex_variable_types() {
        let doc = parse("query($a: [String!]!, $b: [[Int]]) { field }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query($a:[String!]!,$b:[[Int]]){field}"
        );
    }

    #[test]
    fn directive_with_arguments() {
        let doc = parse("{ field @custom(z: 1, a: 2) }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{field@custom(a:0,z:0)}"
        );
    }

    #[test]
    fn multiple_aliased_fields_sorted_by_alias() {
        let doc = parse("{ z: field1 a: field2 m: field3 }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{a:field2 m:field3 z:field1}"
        );
    }

    #[test]
    fn mixed_aliased_and_non_aliased() {
        let doc = parse("{ z: field1 b a: field2 c }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query{a:field2 b c z:field1}"
        );
    }

    #[test]
    fn inline_fragment_tie_breaker_is_canonical() {
        let a_then_b = parse("query { ... on Query { a } ... on Query { b } }");
        let b_then_a = parse("query { ... on Query { b } ... on Query { a } }");

        assert_eq!(normalize(&a_then_b, None), normalize(&b_then_a, None));
    }

    #[test]
    fn aliased_field_tie_breaker_is_canonical() {
        let first = parse("query { x: b x: a }");
        let second = parse("query { x: a x: b }");

        assert_eq!(normalize(&first, None), normalize(&second, None));
    }

    #[test]
    fn error_no_operations() {
        let doc = parse("fragment F on Query { a }");
        assert_eq!(normalize(&doc, None), Err(SignatureError::NoOperations));
    }

    #[test]
    fn float_normalized_to_zero() {
        let doc = parse("{ field(arg: 3.14) }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{field(arg:0)}");
    }

    #[test]
    fn fragment_with_directives() {
        let doc = parse(
            "query { ...F }
            fragment F on Query @deprecated { a }",
        );
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "fragment F on Query@deprecated{a}query{...F}"
        );
    }

    #[test]
    fn variable_definition_directives() {
        let doc = parse("query($x: String @deprecated) { field }");
        assert_eq!(
            normalize(&doc, None).unwrap(),
            "query($x:String@deprecated){field}"
        );
    }

    #[test]
    fn duplicate_fields_preserved() {
        let doc = parse("{ a a a }");
        assert_eq!(normalize(&doc, None).unwrap(), "query{a a a}");
    }

    #[test]
    fn nested_object_default_value() {
        let doc = parse(r#"query($x: Input = { nested: { key: "val" } }) { field }"#);
        assert_eq!(
            normalize(&doc, None).unwrap(),
            r#"query($x:Input={nested:{key:""}}){field}"#
        );
    }
}
