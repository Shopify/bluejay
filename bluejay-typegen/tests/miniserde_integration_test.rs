#![cfg(feature = "miniserde")]

use bluejay_typegen::typegen;
use mnsrd::json;

#[typegen([
    type Query {
        myType: MyType
        myUnion: MyUnion
        myInterface: MyInterface
        builtinScalars: BuiltinScalars
    }
    type MyType implements MyInterface {
        myField: String!
        myList: [String!]!
    }
    type MyOtherType {
        myOtherField: MyScalar!
    }
    type BuiltinScalars {
        int: Int!
        float: Float!
        string: String!
        boolean: Boolean!
        id: ID!
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
    input BuiltinScalarsInput {
        int: Int!
        float: Float!
        string: String!
        boolean: Boolean!
        id: ID!
    }
    scalar MyScalar
    interface MyInterface {
        myField: String!
    }
], codec = "miniserde")]
mod schema {
    type MyScalar = String;

    #[query([
        query Object {
            myType {
                myField
                myList
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

        query Interface {
            myInterface {
                myField
            }
        }

        query ObjectWithFragment {
            myType {
                ...MyFragment
            }
        }

        fragment MyFragment on MyType {
            myField
        }

        query BuiltinScalars {
            builtinScalars {
                int
                float
                string
                boolean
                id
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
fn test_circular_input_object() {
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
fn test_builtin_scalars_input_object() {
    let value = schema::BuiltinScalarsInput {
        int: 1,
        float: 1.2,
        string: "hello".into(),
        boolean: true,
        id: "1".into(),
    };
    assert_eq!(
        "{\"int\":1,\"float\":1.2,\"string\":\"hello\",\"boolean\":true,\"id\":\"1\"}",
        json::to_string(&value)
    );
}

#[test]
fn test_deserialize_object() {
    let raw =
        json::from_str("{\"myType\":{\"myField\":\"hello\",\"myList\":[\"hello\",\"world\"]}}")
            .expect("Error parsing value");
    assert_eq!(
        schema::query::Object {
            my_type: Some(schema::query::object::MyType {
                my_field: "hello".into(),
                my_list: vec!["hello".into(), "world".into()],
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

#[test]
fn test_deserialize_interface() {
    let raw =
        json::from_str("{\"myInterface\":{\"myField\":\"hello\"}}").expect("Error parsing value");
    assert_eq!(
        schema::query::Interface {
            my_interface: Some(schema::query::interface::MyInterface {
                my_field: "hello".into()
            }),
        },
        raw
    );
}

#[test]
fn test_deserialize_object_with_fragment() {
    let raw = json::from_str("{\"myType\":{\"myField\":\"hello\"}}").expect("Error parsing value");
    assert_eq!(
        schema::query::ObjectWithFragment {
            my_type: Some(schema::query::MyFragment {
                my_field: "hello".into(),
            }),
        },
        raw
    );
}

#[test]
fn test_deserialize_builtin_scalars() {
    let raw = json::from_str(
        "{\"builtinScalars\":{\"int\":1,\"float\":1.2,\"string\":\"hello\",\"boolean\":true,\"id\":\"1\"}}",
    )
    .expect("Error parsing value");
    assert_eq!(
        schema::query::BuiltinScalars {
            builtin_scalars: Some(schema::query::builtin_scalars::BuiltinScalars {
                int: 1,
                float: 1.2,
                string: "hello".into(),
                boolean: true,
                id: "1".into(),
            }),
        },
        raw
    );
}
