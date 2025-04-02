use bluejay_typegen_codegen::{generate_schema, Input};
use quote::ToTokens;
use syn::parse_macro_input;

/// Generates Rust types from GraphQL schema definitions and queries.
///
/// ### Arguments
///
/// **Positional:**
///
/// 1. String literal with path to the file containing the schema definition. If relative, should be with respect to
///    the project root (wherever `Cargo.toml` is located). Alternatively, include the schema contents enclosed in square
///    brackets.
///
/// **Optional keyword:**
///
/// _borrow_: Boolean literal indicating whether the generated types should borrow where possible. Defaults to `false`.
/// When `true`, deserializing must be done from a string as a opposed to `serde_json::Value` or a reader.
///
/// ### Trait implementations
///
/// By default, will implement `PartialEq`, `Eq`, `Clone`, and `Debug` for all types. Will implement `Copy` for enums.
/// For types corresponding to values returned from queries, `serde::Deserialize` is implemented. For types that would
/// be arguments to a query, `serde::Serialize` is implemented.
///
/// ### Usage
///
/// Must be used with a module. Inside the module, type aliases must be defined for any custom scalars in the schema.
/// To use a query, define a module within the aforementioned module, and annotate it with
/// `#[query("path/to/query.graphql")]`, where the argument is a string literal path to the query document, or the
/// query contents enclosed in square brackets.
///
/// ### Naming
///
/// To generate idiomatic Rust code, some renaming of types, enum variants, and fields is performed. Types are
/// renamed with `PascalCase`, as are enum variants. Fields are renamed with `snake_case`.
///
/// ### Query restrictions
///
/// In order to keep the type generation code relatively simple, there are some restrictions on the queries that are
/// permitted. This may be relaxed in future versions.
/// * Selection sets on object and interface types must contain either a single fragment spread, or entirely field
///   selections.
/// * Selection sets on union types must contain either a single fragment spread, or both an unaliased `__typename`
///   selection and inline fragments for all or a subset of the objects contained in the union.
///
/// ### Example
/// See top-level documentation of `bluejay-typegen` for an example.
#[proc_macro_attribute]
pub fn typegen(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(attr as Input);
    let mut module = parse_macro_input!(item as syn::ItemMod);

    if let Err(error) = generate_schema(input, &mut module, Default::default()) {
        return error.to_compile_error().into();
    }

    module.to_token_stream().into()
}
