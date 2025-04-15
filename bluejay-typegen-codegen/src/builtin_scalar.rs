use crate::types::string;
use bluejay_core::BuiltinScalarDefinition;
use syn::parse_quote;

pub(crate) fn builtin_scalar_type(scalar: BuiltinScalarDefinition, borrows: bool) -> syn::TypePath {
    match scalar {
        BuiltinScalarDefinition::Boolean => parse_quote! { ::std::primitive::bool },
        BuiltinScalarDefinition::Float => parse_quote! { ::std::primitive::f64 },
        BuiltinScalarDefinition::ID => string(borrows),
        BuiltinScalarDefinition::Int => parse_quote! { ::std::primitive::i32 },
        BuiltinScalarDefinition::String => string(borrows),
    }
}

pub(crate) fn scalar_is_reference(scalar: BuiltinScalarDefinition) -> bool {
    matches!(
        scalar,
        BuiltinScalarDefinition::ID | BuiltinScalarDefinition::String
    )
}
