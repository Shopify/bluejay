#[bluejay_typegen::typegen([
    type Query {
        myType(arg: String): MyType
    }

    type MyType {
        field: String
    }
])]
mod schema {
    #[query([
        query Foo($arg: String) {
            myType(arg: $arg) {
                field
            }
        }

        query FooVariables {
            myType {
                field
            }
        }
    ])]
    mod query {}
}

fn main() {}
