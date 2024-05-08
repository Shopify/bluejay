#[bluejay_typegen::typegen([
    type Query {
        foo: String
    }
])]
mod schema {
    #[query([
        {
            bar
        }
    ])]
    mod query {}
}

fn main() {}
