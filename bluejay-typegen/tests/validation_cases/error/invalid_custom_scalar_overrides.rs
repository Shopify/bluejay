type MyScalarOverride = String;

#[bluejay_typegen::typegen([
    type Query {
        myField: Int
    }
])]
mod schema {
    #[query([
        query MyQuery {
            myField
            myOtherField: myField
        }
    ], custom_scalar_overrides = {
        "MyQuery.myField" => super::MyScalarOverride,
        "MyQuery.myOtherField" => ::std::primitive::i32,
    })]
    mod query {}
}

fn main() {}
