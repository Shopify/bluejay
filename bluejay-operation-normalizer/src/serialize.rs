//! Serializes the normalized IR to a compact canonical string (algorithm step 3).
//!
//! Output format:
//! - Operation type keyword, no operation name, no variable definitions.
//! - All argument and directive values replaced with `$_`.
//! - No whitespace except single spaces separating selections within `{ }`.
//! - Example: `query{field(a:$_,b:$_)@dir{nested}}`

use bluejay_core::OperationType;

use crate::ir::{
    NormalizedDirective, NormalizedField, NormalizedInlineFragment, NormalizedSelection,
};

/// Serialize a normalized operation to its canonical string form (step 3).
pub(crate) fn serialize(
    op_type: OperationType,
    directives: &[NormalizedDirective<'_, '_>],
    selections: &[NormalizedSelection<'_, '_>],
) -> String {
    let mut out = String::with_capacity(256);
    out.push_str(match op_type {
        OperationType::Query => "query",
        OperationType::Mutation => "mutation",
        OperationType::Subscription => "subscription",
    });
    write_directives(&mut out, directives);
    write_selection_set(&mut out, selections);
    out
}

fn write_selection_set(out: &mut String, selections: &[NormalizedSelection<'_, '_>]) {
    out.push('{');
    for (i, sel) in selections.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        match sel {
            NormalizedSelection::Field(f) => write_field(out, f),
            NormalizedSelection::InlineFragment(inf) => write_inline_fragment(out, inf),
        }
    }
    out.push('}');
}

fn write_field(out: &mut String, field: &NormalizedField<'_, '_>) {
    out.push_str(field.name);
    write_arguments(out, &field.arg_names);
    write_directives(out, &field.directives);
    if !field.selections.is_empty() {
        write_selection_set(out, &field.selections);
    }
}

fn write_inline_fragment(out: &mut String, inf: &NormalizedInlineFragment<'_, '_>) {
    out.push_str("...");
    if let Some(tc) = inf.type_condition {
        out.push_str("on ");
        out.push_str(tc);
    }
    write_directives(out, &inf.directives);
    write_selection_set(out, &inf.selections);
}

fn write_directives(out: &mut String, directives: &[NormalizedDirective<'_, '_>]) {
    for dir in directives {
        out.push('@');
        out.push_str(dir.name);
        write_arguments(out, &dir.arg_names);
    }
}

fn write_arguments(out: &mut String, arg_names: &[&str]) {
    if arg_names.is_empty() {
        return;
    }
    out.push('(');
    for (i, name) in arg_names.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(name);
        out.push_str(":$_");
    }
    out.push(')');
}
