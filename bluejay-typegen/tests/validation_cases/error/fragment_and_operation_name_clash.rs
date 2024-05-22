#[bluejay_typegen::typegen([
    type Query {
        myType: MyType
    }

    type MyType {
        field: String
    }
])]
mod schema {
    #[query([
        query Foo {
            myType {
                ...Foo
            }
        }

        fragment Foo on MyType {
            field
        }
    ])]
    mod query {}
}

fn main() {}
