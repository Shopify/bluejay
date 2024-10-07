#[bluejay_typegen::typegen([
    type Query {
        foo: String
    }
], enums_as_str = ["Foo"])]
mod schema {}

fn main() {}
