type StringCow<'a> = std::borrow::Cow<'a, str>;

#[bluejay_typegen::typegen([
    scalar MyScalar

    type Query {
        myField(input: MyInput, oneOfInput: MyOneOfInput): Int
    }

    input MyInput {
        myScalar: MyScalar
        myScalarList: [MyScalar!]!
        myBorrowedScalar: MyScalar!
    }

    input MyOneOfInput @oneOf {
        myScalar: MyScalar
    }
], borrow = true, custom_scalar_overrides = {
    "MyInput.myScalar" => ::std::primitive::i32,
    "MyInput.myScalarList" => ::std::primitive::i32,
    "MyInput.myBorrowedScalar" => super::StringCow<'a>,
    "MyOneOfInput.myScalar" => ::std::primitive::i32,
})]
pub mod schema {
    type MyScalar = String;
}

fn main() {
    let _ = schema::MyInput {
        my_scalar: Some(1),
        my_scalar_list: vec![1],
        my_borrowed_scalar: "hello".into(),
    };
    let _ = schema::MyOneOfInput::MyScalar(1);
}
