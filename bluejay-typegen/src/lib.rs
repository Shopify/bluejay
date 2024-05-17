//! `bluejay-typegen` is a crate for generating Rust types from GraphQL schemas and queries.
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
//!     - Contain an unaliased field selection of `__typename`, and no other field selections, and
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
