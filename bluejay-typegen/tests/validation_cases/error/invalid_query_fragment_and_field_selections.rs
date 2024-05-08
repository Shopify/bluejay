#[bluejay_typegen::typegen([
    type Query {
        foo: String
    }
])]
mod schema {
    #[query([
        {
            foo
            ...MyFragment
        }

        fragment MyFragment on Query {
            foo
        }
    ])]
    mod query {}
}

fn main() {}
