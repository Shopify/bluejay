# `bluejay-typegen`

`bluejay-typegen` provides type generation from GraphQL schemas and executable documents.

## Contributing

`tests/validation_test.rs` makes use of [`trybuild`](https://github.com/dtolnay/trybuild), which is essentially snapshot testing of the compile errors from the code in `tests/validation_cases/error`. To overwrite the snapshots, use the `TRYBUILD=overwrite` environment variable when running `cargo test`.
