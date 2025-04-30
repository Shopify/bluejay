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
        }
    ], custom_scalar_overrides = { "MyQuery.myField" => super::MyScalarOverride })]
    mod query {}
}

fn main() {}
