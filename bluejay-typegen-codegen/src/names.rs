use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::format_ident;

pub(crate) const ANONYMOUS_OPERATION_STRUCT_NAME: &str = "Root";

pub fn type_name(graphql_name: &str) -> String {
    graphql_name.to_case(Case::Pascal)
}

pub fn type_ident(graphql_name: &str) -> Ident {
    to_ident(&type_name(graphql_name))
}

pub fn enum_variant_name(graphql_name: &str) -> String {
    graphql_name.to_case(Case::Pascal)
}

pub fn enum_variant_ident(graphql_name: &str) -> Ident {
    to_ident(&enum_variant_name(graphql_name))
}

pub fn field_name(graphql_name: &str) -> String {
    graphql_name.to_case(Case::Snake)
}

pub fn field_ident(graphql_name: &str) -> Ident {
    to_ident(&field_name(graphql_name))
}

pub fn module_name(graphql_name: &str) -> String {
    graphql_name.to_case(Case::Snake)
}

pub fn module_ident(graphql_name: &str) -> Ident {
    to_ident(&module_name(graphql_name))
}

fn to_ident(name: &str) -> Ident {
    match name {
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move"
        | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await"
        | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" | "union" => {
            format_ident!("r#{}", name)
        }
        _ => format_ident!("{}", name),
    }
}
