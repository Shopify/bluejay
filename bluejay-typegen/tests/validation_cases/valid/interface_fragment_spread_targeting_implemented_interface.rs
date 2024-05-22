#[bluejay_typegen::typegen([
    type Query {
        myInterface: MyInterface
    }

    interface NestedInterface {
        field: String
    }

    interface MyInterface implements NestedInterface {
        field: String
    }

    type MyObject implements MyInterface & NestedInterface {
        field: String
    }
])]
mod schema {
    #[query([
        {
            myInterface {
                ...MyFragment
            }
        }

        fragment MyFragment on NestedInterface {
            field
        }
    ])]
    mod query {}
}

fn main() {}
