#![cfg(feature = "miniserde")]

use bluejay_typegen::typegen;
use mnsrd::json;

#[typegen([
    type Query {
        myNestedField: MyType
        myUnion: MyUnion
    }
    type MyType {
        myField: String!
    }
    type MyOtherType {
        myOtherField: MyScalar!
    }
    union MyUnion = MyType | MyOtherType
    enum MyEnum {
        VARIANT_1
        VARIANT_2
    }
    input MyOneOfInput @oneOf {
        myString: String
        myInt: Int
    }
    input MyInput {
        myField: String
        myCircularField: MyInput
    }
    scalar MyScalar
], codec = "miniserde")]
mod schema {
    type MyScalar = String;

    #[query([
        query Object {
            myNestedField {
                myField
            }
        }

        query Union {
            myUnion {
                __typename
                ...on MyType {
                    myField
                }
                ...on MyOtherType {
                    myOtherField
                }
            }
        }
    ])]
    pub mod query {}
}

#[test]
fn test_enum_deserialization() {
    let raw = json::from_str("\"VARIANT_2\"").expect("Error parsing value");
    assert_eq!(schema::MyEnum::Variant2, raw);

    // Value `UNKNOWN` is not defined in the schema, but it is a potentially
    // valid and non-breaking change that could happen to the schema in the future.
    let raw = json::from_str("\"UNKNOWN\"").expect("Error parsing value");
    assert_eq!(schema::MyEnum::Other, raw);
}

#[test]
fn test_enum_serialization() {
    let value = schema::MyEnum::Variant1;
    let dumped = json::to_string(&value);
    assert_eq!("\"VARIANT_1\"", dumped);
}

#[test]
fn test_one_of_input_object() {
    let value = schema::MyOneOfInput::MyInt(1);
    assert_eq!("{\"myInt\":1}", json::to_string(&value));
    let value = schema::MyOneOfInput::MyString("hello".into());
    assert_eq!("{\"myString\":\"hello\"}", json::to_string(&value));
}

#[test]
fn test_input_object() {
    let value = schema::MyInput {
        my_field: Some("hello".into()),
        my_circular_field: Some(Box::new(schema::MyInput {
            my_field: Some("world".into()),
            my_circular_field: None,
        })),
    };
    assert_eq!(
        "{\"myField\":\"hello\",\"myCircularField\":{\"myField\":\"world\",\"myCircularField\":null}}",
        json::to_string(&value)
    );
}

#[test]
fn test_deserialize_object() {
    let raw =
        json::from_str("{\"myNestedField\":{\"myField\":\"hello\"}}").expect("Error parsing value");
    assert_eq!(
        schema::query::Object {
            my_nested_field: Some(schema::query::object::MyNestedField {
                my_field: "hello".into()
            }),
        },
        raw
    );
}

#[test]
fn test_deserialize_union() {
    let raw = json::from_str("{\"myUnion\":{\"__typename\":\"MyType\",\"myField\":\"hello\"}}")
        .expect("Error parsing value");
    assert_eq!(
        schema::query::Union {
            my_union: Some(schema::query::r#union::MyUnion::MyType {
                my_field: "hello".into()
            }),
        },
        raw
    );
}

#[test]
fn test_deserialize_union_other() {
    // __typename of `Unknown` is not defined in the schema, but it is a potentially
    // valid and non-breaking change that could happen to the schema in the future.
    let raw =
        json::from_str("{\"myUnion\":{\"__typename\":\"Unknown\"}}").expect("Error parsing value");
    assert_eq!(
        schema::query::Union {
            my_union: Some(schema::query::r#union::MyUnion::Other),
        },
        raw
    );
}
