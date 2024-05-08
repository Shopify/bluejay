#[bluejay_typegen::typegen([
    type Query {
        foo: Foo
    }

    scalar Foo
])]
mod schema {}

fn main() {}
