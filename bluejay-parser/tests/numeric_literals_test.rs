use bluejay_core::{AsIter, IntegerValue, Value as _, ValueReference};
use bluejay_parser::{
    ast::{
        definition::{DefaultContext, DefinitionDocument},
        executable::{ExecutableDocument, Selection},
        Parse, Value, VariableValue,
    },
    HasSpan,
};

#[test]
fn integer_literals_are_lossless() {
    let literals = [
        "0".to_owned(),
        "-0".to_owned(),
        "2147483647".to_owned(),
        "-2147483648".to_owned(),
        "2147483648".to_owned(),
        "-2147483649".to_owned(),
        "9007199254740993".to_owned(),
        "9223372036854775808".to_owned(),
        "18446744073709551616".to_owned(),
        "9".repeat(100_000),
        format!("-{}", "9".repeat(100_000)),
    ];
    for literal in literals {
        let source = format!("{{ f(a: {literal}, next: true) }}");
        let parsed = ExecutableDocument::parse(&source);
        let document = parsed.result.expect("valid integer syntax");
        assert_eq!(11, parsed.token_count);
        let Selection::Field(field) = document.operation_definitions()[0]
            .selection_set()
            .iter()
            .next()
            .unwrap()
        else {
            panic!("expected a field");
        };
        let mut arguments = field.arguments().unwrap().iter();
        let value = arguments.next().unwrap().value();
        let Value::Integer(token) = value else {
            panic!("expected an integer token");
        };
        assert_eq!(literal, token.as_ref());
        assert!(std::ptr::eq(
            token.as_ref(),
            &source[value.span().byte_range()]
        ));
        let ValueReference::Integer(integer) = value.as_ref() else {
            panic!("expected an integer");
        };
        assert_eq!(literal, integer.to_string());
        assert_eq!(7..7 + literal.len(), value.span().byte_range());
        assert_eq!("next", arguments.next().unwrap().name().as_ref());
        assert!(arguments.next().is_none());
    }
}

#[test]
fn integer_core_views_compare_numerically() {
    let zero = VariableValue::parse("0").result.unwrap();
    let negative_zero = VariableValue::parse("-0").result.unwrap();
    assert_eq!(zero.as_ref(), negative_zero.as_ref());

    let Value::Integer(token) = negative_zero else {
        panic!("expected an integer token");
    };
    let integer = IntegerValue::from(token);
    assert_eq!(IntegerValue::from(0), integer);
    assert_eq!("-0", integer.to_string());
}

#[test]
fn large_integers_parse_in_defaults_and_nested_values() {
    let source =
        "query Q($id: ID = 18446744073709551616) { f(a: { ids: [$id, -18446744073709551616] }) }";
    assert!(ExecutableDocument::parse(source).result.is_ok());
    let schema = "type Query { f(id: ID = 18446744073709551616): Boolean }";
    assert!(DefinitionDocument::<DefaultContext>::parse(schema)
        .result
        .is_ok());
}

#[test]
fn numeric_literals_reject_unicode_digits() {
    for digit in ["١", "１", "𝟙"] {
        for literal in [
            digit.to_owned(),
            format!("-{digit}"),
            format!("1{digit}"),
            format!("-1{digit}"),
            format!("1{digit}.0"),
            format!("1.{digit}"),
            format!("1.0{digit}"),
            format!("1e{digit}"),
            format!("1e+{digit}"),
            format!("1e1{digit}"),
            format!("1.0e-{digit}"),
            format!("1.0e1{digit}"),
        ] {
            let source = format!("{{ f(a: {literal}) }}");
            assert!(
                ExecutableDocument::parse(&source).result.is_err(),
                "{source}"
            );
        }
    }
}

#[test]
fn removing_integer_range_limits_does_not_relax_number_syntax() {
    for literal in ["01", "-01", "+1", "--1", "1a", "1_0", "1١"] {
        let source = format!("{{ f(a: {literal}) }}");
        assert!(
            ExecutableDocument::parse(&source).result.is_err(),
            "{source}"
        );
    }
}
