#[bluejay_typegen::typegen([
    type Query {
        myInterface: MyInterface
    }

    interface MyInterface {
        field: String
    }

    type MyObject implements MyInterface {
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

        fragment MyFragment on MyObject {
            field
        }
    ])]
    mod query {}
}

fn main() {}
