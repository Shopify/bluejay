#[bluejay_typegen::typegen([
    type Query {
        foo: String
    }
])]
mod schema {
    type Foo = String;
}

fn main() {}
