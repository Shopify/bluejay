#[bluejay_typegen::typegen([
    scalar Foo

    type Query {
        foo: Foo
    }
], enums_as_str = ["Foo"])]
mod schema {
    type Foo = String;
}

fn main() {}
