#![cfg(feature = "miniserde")]

use bluejay_typegen::typegen;
use miniserde::json;

#[typegen([
    type Query {
        myNestedField: MyType
        myUnion: MyUnion
    }
    type MyType {
        myField: String!
    }
    type MyOtherType {
        myOtherField: Int!
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
], codec = "miniserde")]
mod schema {
    #[query([
        {
            myNestedField {
                myField
            }
            myUnion {
                __typename
                ... on MyType {
                    myField
                }
                ... on MyOtherType {
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
}

#[test]
fn test_enum_serialization() {
    let value = schema::MyEnum::Variant1;
    let dumped = json::to_string(&value);
    assert_eq!("\"VARIANT_1\"", dumped);
}

// #[test]
// fn test_one_of_input_object() {
//     let value = schema::MyOneOfInput::MyInt(1);
//     assert_eq!("{\"myInt\":1}", json::to_string(&value));
// }

// #[test]
// fn test_deserialize_union() {
//     let raw = json::from_str("{\"__typename\":\"MyType\",\"myField\":\"hello\"}")
//         .expect("Error parsing value");
//     assert_eq!(
//         schema::query::root::MyUnion::MyType {
//             my_field: "hello".into()
//         },
//         raw
//     );
// }
