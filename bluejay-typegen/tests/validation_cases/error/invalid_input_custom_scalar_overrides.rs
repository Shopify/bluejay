type StringCow<'a> = std::borrow::Cow<'a, str>;

#[bluejay_typegen::typegen([
    scalar MyScalar

    type Query {
        myField(input: MyInput): MyScalar
    }

    input MyInput {
        myField: Int
        myScalarField: MyScalar
    }
], custom_scalar_overrides = {
    "MyInput.myField" => ::std::primitive::i32,
    "MyInput.myMissingField" => ::std::primitive::i32,
    "Query.myField" => ::std::primitive::i32,
    "MyInput.myScalarField" => super::StringCow<'a>,
})]
mod schema {
    type MyScalar = String;
}

fn main() {}
