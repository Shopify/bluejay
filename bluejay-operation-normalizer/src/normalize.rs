use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinition, OperationDefinitionReference, Selection, SelectionReference,
    VariableDefinition, VariableType, VariableTypeReference,
};
use bluejay_core::{
    Argument, AsIter, Directive, ObjectValue, OperationType, Value, ValueReference, Variable,
};
use std::collections::HashMap;

use crate::fragments::collect_used_fragments;
use crate::sort::SelectionSortKey;
use crate::SignatureError;

pub(crate) fn normalize_doc<E: ExecutableDocument>(
    doc: &E,
    op_name: Option<&str>,
) -> Result<String, SignatureError> {
    let fragment_defs: HashMap<&str, &E::FragmentDefinition> =
        doc.fragment_definitions().map(|f| (f.name(), f)).collect();

    let operation = resolve_operation::<E>(doc, op_name)?;
    let op_ref = operation.as_ref();
    let used_fragments = collect_used_fragments::<E>(op_ref.selection_set(), &fragment_defs);

    let mut output = String::with_capacity(256);

    for frag_name in &used_fragments {
        if let Some(frag_def) = fragment_defs.get(frag_name) {
            write_fragment_definition::<E>(&mut output, frag_def);
        }
    }

    write_operation::<E>(&mut output, &op_ref);

    Ok(output)
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

// =====================================================================
// Writing functions
// =====================================================================

fn write_operation<E: ExecutableDocument>(
    out: &mut String,
    op_ref: &OperationDefinitionReference<'_, E::OperationDefinition>,
) {
    out.push_str(match op_ref.operation_type() {
        OperationType::Query => "query",
        OperationType::Mutation => "mutation",
        OperationType::Subscription => "subscription",
    });

    if let Some(name) = op_ref.name() {
        out.push(' ');
        out.push_str(name);
    }

    if let Some(var_defs) = op_ref.variable_definitions() {
        write_variable_definitions::<E>(out, var_defs);
    }

    if let Some(directives) = op_ref.directives() {
        write_directives::<false, E>(out, directives);
    }

    write_selection_set::<E>(out, op_ref.selection_set());
}

fn write_fragment_definition<E: ExecutableDocument>(
    out: &mut String,
    frag: &E::FragmentDefinition,
) {
    out.push_str("fragment ");
    out.push_str(frag.name());
    out.push_str(" on ");
    out.push_str(frag.type_condition());

    if let Some(directives) = frag.directives() {
        write_directives::<false, E>(out, directives);
    }

    write_selection_set::<E>(out, frag.selection_set());
}

fn write_selection_set<E: ExecutableDocument>(out: &mut String, ss: &E::SelectionSet) {
    let mut selections: Vec<&E::Selection> = ss.iter().collect();

    if selections.len() > 1 {
        selections
            .sort_unstable_by(|a, b| sort_key::<E>(&a.as_ref()).cmp(&sort_key::<E>(&b.as_ref())));

        // Tie-break groups with equal sort keys via compact rendering
        let mut group_start = 0;
        while group_start < selections.len() {
            let start_key = sort_key::<E>(&selections[group_start].as_ref());
            let mut group_end = group_start + 1;
            while group_end < selections.len()
                && sort_key::<E>(&selections[group_end].as_ref()) == start_key
            {
                group_end += 1;
            }
            if group_end - group_start > 1 {
                selections[group_start..group_end]
                    .sort_by_cached_key(|sel| render_selection_compact::<E>(sel));
            }
            group_start = group_end;
        }
    }

    out.push('{');
    for (i, sel) in selections.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        write_selection::<E>(out, &sel.as_ref());
    }
    out.push('}');
}

fn sort_key<'a, E: ExecutableDocument>(
    sel: &SelectionReference<'a, E::Selection>,
) -> SelectionSortKey<'a> {
    match sel {
        SelectionReference::Field(f) => SelectionSortKey::Field(f.alias().unwrap_or(f.name())),
        SelectionReference::FragmentSpread(s) => SelectionSortKey::FragmentSpread(s.name()),
        SelectionReference::InlineFragment(i) => {
            SelectionSortKey::InlineFragment(i.type_condition())
        }
    }
}

fn render_selection_compact<E: ExecutableDocument>(sel: &E::Selection) -> String {
    let mut out = String::with_capacity(64);
    write_selection::<E>(&mut out, &sel.as_ref());
    out
}

fn write_selection<E: ExecutableDocument>(
    out: &mut String,
    sel: &SelectionReference<'_, E::Selection>,
) {
    match sel {
        SelectionReference::Field(f) => write_field::<E>(out, f),
        SelectionReference::FragmentSpread(s) => write_fragment_spread::<E>(out, s),
        SelectionReference::InlineFragment(i) => write_inline_fragment::<E>(out, i),
    }
}

fn write_field<E: ExecutableDocument>(out: &mut String, field: &E::Field) {
    if let Some(alias) = field.alias() {
        out.push_str(alias);
        out.push(':');
    }
    out.push_str(field.name());

    if let Some(args) = field.arguments() {
        write_arguments::<false, E>(out, args);
    }

    if let Some(directives) = field.directives() {
        write_directives::<false, E>(out, directives);
    }

    if let Some(ss) = field.selection_set() {
        write_selection_set::<E>(out, ss);
    }
}

fn write_fragment_spread<E: ExecutableDocument>(out: &mut String, spread: &E::FragmentSpread) {
    out.push_str("...");
    out.push_str(spread.name());

    if let Some(directives) = spread.directives() {
        write_directives::<false, E>(out, directives);
    }
}

fn write_inline_fragment<E: ExecutableDocument>(out: &mut String, inline: &E::InlineFragment) {
    out.push_str("...");
    if let Some(tc) = inline.type_condition() {
        out.push_str(" on ");
        out.push_str(tc);
    }

    if let Some(directives) = inline.directives() {
        write_directives::<false, E>(out, directives);
    }

    write_selection_set::<E>(out, inline.selection_set());
}

fn write_variable_definitions<E: ExecutableDocument>(
    out: &mut String,
    var_defs: &E::VariableDefinitions,
) {
    let mut vars: Vec<_> = var_defs.iter().collect();
    if vars.is_empty() {
        return;
    }
    vars.sort_unstable_by(|a, b| a.variable().cmp(b.variable()));

    out.push('(');
    for (i, var) in vars.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('$');
        out.push_str(var.variable());
        out.push(':');
        write_variable_type::<E>(out, &var.r#type().as_ref());
        if let Some(default) = var.default_value() {
            out.push('=');
            write_value::<true, E>(out, default);
        }
        if let Some(directives) = var.directives() {
            write_directives::<true, E>(out, directives);
        }
    }
    out.push(')');
}

fn write_variable_type<E: ExecutableDocument>(
    out: &mut String,
    vt: &VariableTypeReference<'_, E::VariableType>,
) {
    match vt {
        VariableTypeReference::Named(name, required) => {
            out.push_str(name);
            if *required {
                out.push('!');
            }
        }
        VariableTypeReference::List(inner, required) => {
            out.push('[');
            write_variable_type::<E>(out, &inner.as_ref());
            out.push(']');
            if *required {
                out.push('!');
            }
        }
    }
}

fn write_directives<const CONST: bool, E: ExecutableDocument>(
    out: &mut String,
    directives: &E::Directives<CONST>,
) {
    let mut dirs: Vec<_> = directives.iter().collect();
    dirs.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    for dir in &dirs {
        out.push('@');
        out.push_str(dir.name());
        if let Some(args) = dir.arguments() {
            write_arguments::<CONST, E>(out, args);
        }
    }
}

fn write_arguments<const CONST: bool, E: ExecutableDocument>(
    out: &mut String,
    args: &E::Arguments<CONST>,
) {
    let mut arg_list: Vec<_> = args.iter().collect();
    if arg_list.is_empty() {
        return;
    }
    arg_list.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    out.push('(');
    for (i, arg) in arg_list.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(arg.name());
        out.push(':');
        write_value::<CONST, E>(out, arg.value());
    }
    out.push(')');
}

fn write_value<const CONST: bool, E: ExecutableDocument>(
    out: &mut String,
    value: &E::Value<CONST>,
) {
    match value.as_ref() {
        ValueReference::Variable(var) => {
            out.push('$');
            out.push_str(var.name());
        }
        ValueReference::Integer(_) | ValueReference::Float(_) => out.push('0'),
        ValueReference::String(_) => out.push_str("\"\""),
        ValueReference::Boolean(b) => out.push_str(if b { "true" } else { "false" }),
        ValueReference::Null => out.push_str("null"),
        ValueReference::Enum(e) => out.push_str(e),
        ValueReference::List(_) => out.push_str("[]"),
        ValueReference::Object(obj) => {
            let mut entries: Vec<_> = obj.iter().collect();
            entries.sort_unstable_by(|a, b| a.0.as_ref().cmp(b.0.as_ref()));
            out.push('{');
            for (i, (key, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(key.as_ref());
                out.push(':');
                write_value::<CONST, E>(out, val);
            }
            out.push('}');
        }
    }
}
