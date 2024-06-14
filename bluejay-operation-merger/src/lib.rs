//! `bluejay-operation-merger` is a library for merging multiple GraphQL operations into a single operation.
//!
//! # Limitations
//! There is no guarantee that the merged operation will be valid according to the GraphQL specification. The following limitations apply:
//! - operation types of the same name must have the same operation type (query, mutation, or subscription)
//! - directives are not supported
//!
//! # Example
//! ```
//! use bluejay_operation_merger::{MergedExecutableDocument, ExecutableDocumentEntry};
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
//! let user_provided_context = Default::default();
//!
//! let entry1 = ExecutableDocumentEntry::new(&parsed1, &user_provided_context);
//! let entry2 = ExecutableDocumentEntry::new(&parsed2, &user_provided_context);
//!
//! let merged = MergedExecutableDocument::new([entry1, entry2]).unwrap();
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

mod argument;
mod context;
mod directive;
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
mod value;
mod variable_definition;
mod variable_definitions;

use argument::{MergedArgument, MergedArguments};
use context::Context;
use directive::MergedDirective;
use directives::EmptyDirectives;
use error::Error;
pub use executable_document::{ExecutableDocumentEntry, MergedExecutableDocument};
use field::MergedField;
use fragment_definition::MergedFragmentDefinition;
use fragment_spread::MergedFragmentSpread;
use id::{Id, IdGenerator};
use inline_fragment::MergedInlineFragment;
use never::Never;
use operation_definition::MergedOperationDefinition;
use selection::MergedSelection;
use selection_set::MergedSelectionSet;
use value::MergedValue;
use variable_definition::MergedVariableDefinition;
use variable_definitions::MergedVariableDefinitions;
