#[bluejay_typegen::typegen([
    type Query {
        result(result: Result!): Void
        option(option: Option!): Void
        type: String
        fn: String
        impl: String
    }

    input Result @oneOf {
        ok: String
        err: String
    }

    input Option @oneOf {
        some: String
        none: Void
    }

    scalar Void
])]
mod schema {
    type Void = ();

    #[query([
        {
            result(result: { ok: "ok" })
            option(option: { some: "some" })
            type
            fn
            impl
        }
    ])]
    mod query {}
}

fn main() {}
