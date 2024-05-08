#[bluejay_typegen::typegen([
    type Query {
        foo: Bar
    }
])]
mod schema {}

fn main() {}
