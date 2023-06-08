use bluejay_typegen::typegen;

#[typegen("tests/schema.graphql", borrow = true)]
mod my_schema {
    type Decimal<'a> = std::borrow::Cow<'a, str>;
    type UnsignedInt = u32;

    #[query("tests/query.graphql")]
    pub mod query {}
}

#[test]
fn test_enum_deserialization() {
    let raw = serde_json::json!("VARIANT_2");
    let parsed = serde_json::from_value(raw).expect("Error parsing value");
    assert_eq!(my_schema::MyEnum::Variant2, parsed);
}

#[test]
fn test_enum_serialization() {
    let value = my_schema::MyEnum::Variant1;
    let dumped = serde_json::to_value(value).expect("Error serializing value");
    assert_eq!(serde_json::json!("VARIANT_1"), dumped);
}

#[test]
fn test_input_object() {
    let value = my_schema::MyInput {
        my_field: "x".into(),
        my_circular_field: Some(Box::new(my_schema::MyInput {
            my_field: "y".into(),
            my_circular_field: None,
        })),
    };
    assert_eq!(
        serde_json::json!({ "myField": "x", "myCircularField": { "myField": "y", "myCircularField": null } }),
        serde_json::to_value(value).expect("Error serializing value"),
    );
}

#[test]
fn test_one_of_input_object() {
    let value = my_schema::MyOneOfInput::MyInt(1);
    assert_eq!(
        serde_json::json!({ "myInt": 1 }),
        serde_json::to_value(value).expect("Error serializing value"),
    );
}

#[test]
fn test_object_query_deserialization() {
    let value = serde_json::json!({
        "myField": "hello",
        "myAliasedField": "world",
        "myNestedField": {
            "myField": "hello"
        },
        "myRequiredField": "hello",
        "myNestedFieldWithFragment": {
            "myField": "hello"
        },
        "player": {
            "__typename": "Skater",
            "name": "Auston Matthews",
            "stats": [
                {
                    "goals": 60,
                },
            ],
        },
        "type": "hello",
        "myEnum": "VARIANT_1",
        "myDecimals": ["1.2", "3.4"],
    });
    let raw = value.to_string();
    let parsed = serde_json::from_str(&raw).expect("Error parsing value");
    assert_eq!(
        my_schema::query::MyQuery {
            my_field: Some("hello".into()),
            my_aliased_field: Some("world".into()),
            my_nested_field: Some(my_schema::query::my_query::MyNestedField {
                my_field: Some("hello".into())
            }),
            my_required_field: "hello".into(),
            my_nested_field_with_fragment: Some(my_schema::query::MyType {
                my_field: Some("hello".into())
            }),
            player: my_schema::query::my_query::Player::Skater {
                name: "Auston Matthews".into(),
                stats: vec![my_schema::query::my_query::player::skater::Stats { goals: 60 }]
            },
            r#type: Some("hello".into()),
            my_enum: my_schema::MyEnum::Variant1,
            my_decimals: vec!["1.2".into(), "3.4".into()],
        },
        parsed,
    );
    assert!(matches!(
        parsed.my_required_field,
        std::borrow::Cow::Borrowed(_),
    ));
}
