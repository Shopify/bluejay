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
                ...on Bar { bar }
                ...on Baz { baz }
            }
        }
    ])]
    mod query {}
}

fn main() {}
