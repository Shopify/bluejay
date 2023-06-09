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

pub use bluejay_typegen_macro::typegen;
pub use serde;
