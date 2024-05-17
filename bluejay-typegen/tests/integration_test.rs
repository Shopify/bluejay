use bluejay_typegen::typegen;

#[typegen("tests/schema.graphql", borrow = true)]
mod schema {
    type Decimal<'a> = std::borrow::Cow<'a, str>;
    type UnsignedInt = u32;

    #[query("tests/query.graphql")]
    pub mod query {}
}

#[test]
fn test_enum_deserialization() {
    let raw = serde_json::json!("VARIANT_2");
    let parsed = serde_json::from_value(raw).expect("Error parsing value");
    assert_eq!(schema::MyEnum::Variant2, parsed);

    // Value `UNKNOWN` is not defined in the schema, but it is a potentially
    // valid and non-breaking change that could happen to the schema in the future.
    let raw = serde_json::json!("UNKNOWN");
    let parsed = serde_json::from_value(raw).expect("Error parsing value");
    assert_eq!(schema::MyEnum::Other, parsed);
}

#[test]
fn test_enum_serialization() {
    let value = schema::MyEnum::Variant1;
    let dumped = serde_json::to_value(value).expect("Error serializing value");
    assert_eq!(serde_json::json!("VARIANT_1"), dumped);
}

#[test]
fn test_input_object() {
    let value = schema::MyInput {
        my_field: "x".into(),
        my_circular_field: Some(Box::new(schema::MyInput {
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
    let value = schema::MyOneOfInput::MyInt(1);
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
        "type": "hello",
        "myEnum": "VARIANT_1",
        "myDecimals": ["1.2", "3.4"],
    });
    let raw = value.to_string();
    let parsed = serde_json::from_str(&raw).expect("Error parsing value");
    assert_eq!(
        schema::query::MyQuery {
            my_field: Some("hello".into()),
            my_aliased_field: Some("world".into()),
            my_nested_field: Some(schema::query::my_query::MyNestedField {
                my_field: Some("hello".into())
            }),
            my_required_field: "hello".into(),
            my_nested_field_with_fragment: Some(schema::query::MyType {
                my_field: Some("hello".into())
            }),
            r#type: Some("hello".into()),
            my_enum: schema::MyEnum::Variant1,
            my_decimals: vec!["1.2".into(), "3.4".into()],
        },
        parsed,
    );
    assert!(matches!(
        parsed.my_required_field,
        std::borrow::Cow::Borrowed(_),
    ));
}

#[test]
fn test_union_query_deserialization() {
    let value = serde_json::json!({
        "player": {
            "__typename": "Skater",
            "name": "Auston Matthews",
            "age": 25,
            "position": "CENTRE",
            "stats": [
                {
                    "goals": 60
                },
            ],
        },
    })
    .to_string();

    let result: schema::query::Player = serde_json::from_str(&value).expect("Error parsing value");

    assert_eq!(
        schema::query::Player {
            player: schema::query::player::Player::Skater {
                name: "Auston Matthews".into(),
                age: 25,
                position: schema::Position::Centre,
                stats: vec![schema::query::player::player::skater::Stats { goals: 60 }],
            },
        },
        result,
    );
}

#[test]
fn test_union_query_deserialization_other() {
    // __typename of `Unknown` is not defined in the schema, but it is a potentially
    // valid and non-breaking change that could happen to the schema in the future.
    let value = serde_json::json!({
        "player": {
            "__typename": "Unknown",
        },
    })
    .to_string();

    let result: schema::query::Player = serde_json::from_str(&value).expect("Error parsing value");

    assert_eq!(
        schema::query::Player {
            player: schema::query::player::Player::Other,
        },
        result,
    );
}
