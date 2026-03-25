use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, OperationDefinition};
use std::collections::{HashMap, HashSet};

use crate::build::{build_directives, build_selections};
use crate::normalize_ir::normalize_selections;
use crate::serialize::serialize;
use crate::SignatureError;

pub(crate) fn normalize_doc<E: ExecutableDocument>(
    doc: &E,
    op_name: Option<&str>,
) -> Result<String, SignatureError> {
    let fragment_defs: HashMap<&str, &E::FragmentDefinition> =
        doc.fragment_definitions().map(|f| (f.name(), f)).collect();

    let operation = resolve_operation::<E>(doc, op_name)?;
    let op_ref = operation.as_ref();

    let op_directives = build_directives::<false, E>(op_ref.directives());

    let mut expanding = HashSet::new();
    let mut selections =
        build_selections::<E>(op_ref.selection_set(), &fragment_defs, &mut expanding);

    normalize_selections(&mut selections);

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
