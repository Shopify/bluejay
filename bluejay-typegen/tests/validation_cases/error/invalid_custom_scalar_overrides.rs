type MyScalarOverride = String;
type StringCow<'a> = std::borrow::Cow<'a, str>;

#[bluejay_typegen::typegen([
    scalar MyScalar

    type Query {
        myField: Int
        myScalarField: MyScalar
    }
])]
mod schema {
    type MyScalar = String;

    #[query([
        query MyQuery {
            myField
            myOtherField: myField
            myScalarField
        }
    ], custom_scalar_overrides = {
        "MyQuery.myField" => super::MyScalarOverride,
        "MyQuery.myOtherField" => ::std::primitive::i32,
        "MyQuery.myScalarField" => super::StringCow<'a>,
    })]
    mod query {}
}

fn main() {}
