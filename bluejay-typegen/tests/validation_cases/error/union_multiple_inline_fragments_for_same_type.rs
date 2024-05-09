#[bluejay_typegen::typegen([
    type Query {
        foo: Foo
    }

    type Bar {
        bar: String
    }

    type Baz {
        baz: String
    }

    union Foo = Bar | Baz
])]
mod schema {
    #[query([
        {
            foo {
                __typename
                ...on Bar { bar }
                ...on Bar { bar }
            }
        }
    ])]
    mod query {}
}

fn main() {}
