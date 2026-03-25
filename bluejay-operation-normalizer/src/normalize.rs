//! Orchestrates the normalization pipeline (algorithm steps 1–3).
//!
//! 1. Resolves the target operation.
//! 2. Delegates to [`build::build_selections`] for IR construction + normalization.
//! 3. Delegates to [`serialize::serialize`] for canonical string output.

use bumpalo::Bump;
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, OperationDefinition};

use crate::build::{build_directives, build_selections};
use crate::serialize::serialize;
use crate::SignatureError;

/// Entry point: resolve the operation, build normalized IR, serialize to canonical string.
pub(crate) fn normalize_doc<E: ExecutableDocument>(
    doc: &E,
    op_name: Option<&str>,
) -> Result<String, SignatureError> {
    let operation = resolve_operation::<E>(doc, op_name)?;
    let op_ref = operation.as_ref();

    let bump = Bump::new();

    let fragment_defs: Vec<(&str, &E::FragmentDefinition)> =
        doc.fragment_definitions().map(|f| (f.name(), f)).collect();

    let op_directives = build_directives::<false, E>(op_ref.directives(), &bump);

    let mut expanding = Vec::new();
    let selections =
        build_selections::<E>(op_ref.selection_set(), &fragment_defs, &mut expanding, &bump);

    Ok(serialize(
        op_ref.operation_type(),
        &op_directives,
        &selections,
    ))
}

fn resolve_operation<'a, E: ExecutableDocument>(
    doc: &'a E,
    op_name: Option<&str>,
) -> Result<&'a E::OperationDefinition, SignatureError> {
    match op_name {
        Some(name) => doc
            .operation_definitions()
            .find(|op| op.as_ref().name() == Some(name))
            .ok_or_else(|| SignatureError::OperationNotFound(name.to_string())),
        None => {
            let mut iter = doc.operation_definitions();
            let first = iter.next().ok_or(SignatureError::NoOperations)?;
            if iter.next().is_some() {
                return Err(SignatureError::AmbiguousOperation);
            }
            Ok(first)
        }
    }
}
