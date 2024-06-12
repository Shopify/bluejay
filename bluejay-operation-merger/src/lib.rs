//! `bluejay-operation-merger` is a library for merging multiple GraphQL operations into a single operation.
//!
//! # Limitations
//! There is no guarantee that the merged operation will be valid according to the GraphQL specification. The following limitations apply:
//! - operation types of the same name must have the same operation type (query, mutation, or subscription)
//! - directives are not supported
//!
//! # Example
//! ```
//! use bluejay_operation_merger::MergedExecutableDocument;
//! use bluejay_parser::ast::{executable::ExecutableDocument, Parse};
//! use bluejay_printer::executable::ExecutableDocumentPrinter;
//!
//! let s1 = r#"
//! query {
//!   foo {
//!     bar
//!     ... on AbstractType {
//!       x
//!     }
//!   }
//! }
//! "#;
//! let s2 = r#"
//! query {
//!   foo {
//!     baz
//!     ...MyFragment
//!   }
//! }
//!
//! fragment MyFragment on AbstractType {
//!   y
//! }
//! "#;
//!
//! let parsed1 = ExecutableDocument::parse(s1).unwrap();
//! let parsed2 = ExecutableDocument::parse(s2).unwrap();
//!
//! let merged = MergedExecutableDocument::new([&parsed1, &parsed2]).unwrap();
//!
//! let expected = r#"query {
//!   foo {
//!     bar
//!     ...on AbstractType {
//!       x
//!       y
//!     }
//!     baz
//!   }
//! }
//! "#;
//!
//! assert_eq!(
//!   expected,
//!   ExecutableDocumentPrinter::new(&merged).to_string()
//! );
//! ```

mod context;
mod directives;
mod error;
mod executable_document;
mod field;
mod fragment_definition;
mod fragment_spread;
mod id;
mod inline_fragment;
mod never;
mod operation_definition;
mod selection;
mod selection_set;
mod variable_definition;
mod variable_definitions;

use context::Context;
use directives::EmptyDirectives;
use error::Error;
pub use executable_document::MergedExecutableDocument;
use field::MergedField;
use fragment_definition::MergedFragmentDefinition;
use fragment_spread::MergedFragmentSpread;
use id::{Id, IdGenerator};
use inline_fragment::MergedInlineFragment;
use never::Never;
use operation_definition::MergedOperationDefinition;
use selection::MergedSelection;
use selection_set::MergedSelectionSet;
use variable_definition::MergedVariableDefinition;
use variable_definitions::MergedVariableDefinitions;
