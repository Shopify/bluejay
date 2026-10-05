type StringCow<'a> = std::borrow::Cow<'a, str>;

#[bluejay_typegen::typegen([
    scalar MyScalar

    type Query {
        myField(input: MyInput, oneOfInput: MyOneOfInput, count: Int): Int
    }

    input MyInput {
        myScalar: MyScalar
        myScalarList: [MyScalar!]!
        myBorrowedScalar: MyScalar!
        myVariables: MyScalar!
    }

    input MyOneOfInput @oneOf {
        myScalar: MyScalar
    }
], borrow = true, custom_scalar_overrides = {
    "MyInput.myScalar" => ::std::primitive::i32,
    "MyInput.myScalarList" => ::std::primitive::i32,
    "MyInput.myBorrowedScalar" => super::StringCow<'a>,
    "MyInput.myVariables" => query::MyQueryVariables,
    "MyOneOfInput.myScalar" => ::std::primitive::i32,
})]
pub mod schema {
    type MyScalar = String;

    #[query([
        query MyQuery($count: Int!) {
            myField(count: $count)
        }
    ])]
    pub mod query {}
}

fn main() {
    let _ = schema::MyInput {
        my_scalar: Some(1),
        my_scalar_list: vec![1],
        my_borrowed_scalar: "hello".into(),
        my_variables: schema::query::MyQueryVariables { count: 1 },
    };
    let _ = schema::MyOneOfInput::MyScalar(1);
}
