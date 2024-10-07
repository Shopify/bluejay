//! `bluejay-typegen` is a crate for generating Rust types from GraphQL schemas and queries.
//!
//! ### Usage
//! The [`typegen`] macro generates Rust types from a GraphQL schema and any number of queries.
//! The macro must decorate a module, and the module must contain a type alias for each custom scalar defined in the schema.
//! All shared types for the schema (input object, enums) are generated within the module.
//!
//! #### Arguments
//! The macro takes one positional argument, followed by a series of optional named arguments.
//! The positional argument can be either a string literal pointing to a file containing the schema in SDL format (path relative to the `Cargo.toml` of the crate where the macro is used),
//! or DSL code defining the schema directly in the macro invocation, enclosed within square brackets.
//! The optional named arguments are:
//! - `borrow`: A boolean indicating whether the generated types should borrow strings from the input JSON value instead of owning them. Defaults to `false`.
//! - `codec`: A string literal specifying the codec to use for serializing and deserializing values.
//!   Must be one of `"serde"` or `"miniserde"`. Defaults to `"serde"` when the `serde` feature is enabled, otherwise `"miniserde"` when the `miniserde` feature is enabled.
//!   When `"miniserde"` is used, `borrow` must be `false` as `miniserde` does not support borrowing strings.
//! - `enums_as_str`: An array of string literals containing the names of enum types from the GraphQL schema that should be represented as strings. Defaults to `[]`.
//!   When `borrow` is true, the values are `std::borrow::Cow<str>`, otherwise they are `String`.
//!
//! #### Queries
//! Within the module defining the schema definition, a submodule can be defined for any number of executable documents.
//! This can be done by decorating the submodule with `#[query(...)]` where the argument follows the same convention as the positional argument of the macro.
//! For each operation and fragment definition in the query document, a corresponding Rust type is generated. If an anonymous operation is defined, the type is named `Root`.
//! See [type path pattern](#type-path-pattern) for more information on how the path for a given type is determined.
//!
//! ### Example
//! ```
//! #[bluejay_typegen::typegen([
//!   scalar UnsignedInt
//!
//!   enum Position {
//!     WING
//!     CENTRE
//!     DEFENCE
//!   }
//!
//!   type Skater {
//!     name: String!
//!     position: Position!
//!     age: UnsignedInt!
//!     stats: [SkaterStat!]!
//!   }
//!
//!   type SkaterStat {
//!     goals: UnsignedInt!
//!   }
//!
//!   type Goalie {
//!     name: String!
//!     age: UnsignedInt!
//!     stats: [GoalieStat!]!
//!   }
//!
//!   type GoalieStat {
//!     wins: UnsignedInt!
//!   }
//!
//!   union Player = Skater | Goalie
//!
//!   type Query {
//!     player: Player!
//!   }
//! ], borrow = true)]
//! mod schema {
//!     type UnsignedInt = u32;
//!
//!     #[query([
//!       query Player {
//!         player {
//!           __typename
//!           ...on Skater {
//!             name
//!             age
//!             position
//!             stats { goals }
//!           }
//!           ...on Goalie {
//!             name
//!             age
//!             stats { wins }
//!           }
//!         }
//!       }
//!     ])]
//!     pub mod query {}
//! }
//!
//! let value = serde_json::json!({
//!     "player": {
//!         "__typename": "Skater",
//!         "name": "Auston Matthews",
//!         "age": 25,
//!         "position": "CENTRE",
//!         "stats": [
//!             {
//!                 "goals": 60
//!             },
//!         ],
//!     },
//! }).to_string();
//!
//! let result: schema::query::Player = serde_json::from_str(&value).expect("Error parsing value");
//!
//! assert_eq!(
//!     schema::query::Player {
//!         player: schema::query::player::Player::Skater {
//!             name: "Auston Matthews".into(),
//!             age: 25,
//!             position: schema::Position::Centre,
//!             stats: vec![schema::query::player::player::skater::Stats { goals: 60 }],
//!         },
//!     },
//!     result,
//! );
//! ```
//!
//! ### Limitations
//! - A query cannot contain a fragment definition with the same name as an operation definition
//! - A schema module must contain exactly one type alias for each custom scalar defined in the schema, so that the type alias can be used in the generated Rust types
//! - Within the scope of an object type, the selection set must not contain any inline fragments and must either:
//!     - Contain at least one field selection, or
//!     - Contain exactly one fragment spread
//! - Within the scope of an interface type, the selection set must not contain any inline fragments and must either:
//!     - Contain at least one field selection, or
//!     - Contain exactly one fragment spread, where the target type of the fragment spread is either the interface type itself or an interface that the interface type implements
//! - Within the scope of a union type, the selection set must:
//!     - Contain an unaliased field selection of `__typename` as the first selection in the set, and no other field selections, and
//!     - Not contain any fragment spreads, and
//!     - Not contain multiple inline fragments targeting any object type in the union type, and
//!     - Not contain any inline fragments targeting types that are not a member of the union type
//!
//! ### Type path pattern
//! The path for a given type in the generated Rust types is determined by the following rules:
//! - If the type is a custom scalar, enum, or input type, the path is `schema_module::TypeName`. For example, the `Position` enum in the example above has the path `schema::Position`.
//! - If the type is an operation root type, the path is `schema_module::query_module::OperationName`. For example, the `Player` type for the `Player` query root in the example above has the path `schema::query::Player`.
//! - If the type is an anonymous operation root type, the path is `schema_module::query_module::Root`
//! - If the type is a nested object type, the path is nested under the path of the parent object type, like `schema_module::query_module::operation_name::TypeName`. For example, the `Player` Rust enum type for the `player` field in the example above has the path `schema::query::player::Player`. And the `Stats` Rust struct type for the `stats` field in the `Skater` arm of the `Player` enum has the path `schema::query::player::player::skater::Stats`.
//! - If the type is a fragment definition, the path is `schema_module::query_module::FragmentName`, with all nested types following the same pattern as operation types, e.g. at `schema_module::query_module::fragment_name::TypeName`.

pub use bluejay_typegen_macro::typegen;

#[cfg(feature = "serde")]
pub use srd as serde;

#[cfg(feature = "miniserde")]
pub use mnsrd as miniserde;
