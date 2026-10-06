type MyScalarOverride = String;

#[bluejay_typegen::typegen([
    scalar MyScalar

    type Query {
        myField: MyScalar!
    }
])]
pub mod schema {
    type MyScalar = String;

    #[query([
        query MyQuery {
            myField
            myAliasedField: myField
            myOtherAliasedField: myField
        }
    ], custom_scalar_overrides = {
        "MyQuery.myField" => super::MyScalarOverride,
        "MyQuery.myAliasedField" => ::std::primitive::i32,
        "MyQuery.myOtherAliasedField" => (),
    })]
    pub mod query {}
}

fn main() {
    let _ = schema::query::MyQuery {
        my_field: "hello".to_string(),
        my_aliased_field: 1,
        my_other_aliased_field: (),
    };
}
