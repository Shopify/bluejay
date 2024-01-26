use crate::Config;
use bluejay_core::{definition::SchemaDefinition, BuiltinScalarDefinition};
use syn::parse_quote;

pub(crate) fn builtin_scalar_type<S: SchemaDefinition>(
    scalar: BuiltinScalarDefinition,
    config: &Config<S>,
) -> syn::TypePath {
    match scalar {
        BuiltinScalarDefinition::Boolean => parse_quote! { ::std::primitive::bool },
        BuiltinScalarDefinition::Float => parse_quote! { ::std::primitive::f64 },
        BuiltinScalarDefinition::ID => string(config),
        BuiltinScalarDefinition::Int => parse_quote! { ::std::primitive::i32 },
        BuiltinScalarDefinition::String => string(config),
    }
}

pub(crate) fn scalar_is_reference(scalar: BuiltinScalarDefinition) -> bool {
    matches!(
        scalar,
        BuiltinScalarDefinition::ID | BuiltinScalarDefinition::String
    )
}

fn string<S: SchemaDefinition>(config: &Config<S>) -> syn::TypePath {
    if config.borrow() {
        parse_quote! { ::std::borrow::Cow<'a, str> }
    } else {
        parse_quote! { ::std::string::String }
    }
}
