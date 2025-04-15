use bluejay_typegen_codegen::{
    generate_schema, names::field_ident, CodeGenerator, ExecutableEnum, ExecutableField,
    ExecutableStruct, Input, WrappedExecutableType,
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote};

/// Generates Rust types from GraphQL schema definitions and queries.
///
/// ### Arguments
///
/// **Positional:**
///
/// 1. String literal with path to the file containing the schema definition. If relative, should be with respect to
///    the project root (wherever `Cargo.toml` is located). Alternatively, include the schema contents enclosed in square
///    brackets.
///
/// **Optional keyword:**
///
/// _borrow_: Boolean literal indicating whether the generated types should borrow where possible. Defaults to `false`.
/// When `true`, deserializing must be done from a string as a opposed to `serde_json::Value` or a reader.
///
/// ### Trait implementations
///
/// By default, will implement `PartialEq`, `Eq`, `Clone`, and `Debug` for all types. Will implement `Copy` for enums.
/// For types corresponding to values returned from queries, `serde::Deserialize` is implemented. For types that would
/// be arguments to a query, `serde::Serialize` is implemented.
///
/// ### Usage
///
/// Must be used with a module. Inside the module, type aliases must be defined for any custom scalars in the schema.
/// To use a query, define a module within the aforementioned module, and annotate it with
/// `#[query("path/to/query.graphql")]`, where the argument is a string literal path to the query document, or the
/// query contents enclosed in square brackets.
///
/// ### Naming
///
/// To generate idiomatic Rust code, some renaming of types, enum variants, and fields is performed. Types are
/// renamed with `PascalCase`, as are enum variants. Fields are renamed with `snake_case`.
///
/// ### Query restrictions
///
/// In order to keep the type generation code relatively simple, there are some restrictions on the queries that are
/// permitted. This may be relaxed in future versions.
/// * Selection sets on object and interface types must contain either a single fragment spread, or entirely field
///   selections.
/// * Selection sets on union types must contain either a single fragment spread, or both an unaliased `__typename`
///   selection and inline fragments for all or a subset of the objects contained in the union.
///
/// ### Example
/// See top-level documentation of `bluejay-typegen` for an example.
#[proc_macro_attribute]
pub fn typegen(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(attr as Input);
    let mut module = parse_macro_input!(item as syn::ItemMod);

    if let Err(error) = generate_schema(input, &mut module, Default::default(), SerdeCodeGenerator)
    {
        return error.to_compile_error().into();
    }

    module.to_token_stream().into()
}

struct SerdeCodeGenerator;

impl CodeGenerator for SerdeCodeGenerator {
    fn attributes_for_executable_struct(
        &self,
        _executable_struct: &ExecutableStruct,
    ) -> Vec<syn::Attribute> {
        vec![
            parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug, ::bluejay_typegen::serde::Deserialize)] },
            parse_quote! { #[serde(crate = "bluejay_typegen::serde")] },
        ]
    }

    fn fields_for_executable_struct(&self, executable_struct: &ExecutableStruct) -> syn::Fields {
        let fields: Vec<syn::Field> = executable_struct
            .fields()
            .iter()
            .map(|executable_field| {
                let name_ident = field_ident(executable_field.graphql_name());

                let attributes = self.attributes_for_field(executable_field);
                let type_path = executable_struct.type_for_field(executable_field, false);

                parse_quote! {
                    #(#attributes)*
                    pub #name_ident: #type_path
                }
            })
            .collect();

        let fields_named: syn::FieldsNamed = parse_quote! { { #(#fields,)* } };

        fields_named.into()
    }

    fn field_accessor_block(
        &self,
        _executable_struct: &ExecutableStruct,
        field: &ExecutableField,
    ) -> syn::Block {
        let name_ident = field_ident(field.graphql_name());

        let expr = Self::reference_property_for_type(field.r#type(), &name_ident);

        parse_quote! {
            {
                #expr
            }
        }
    }

    fn attributes_for_executable_enum(
        &self,
        _executable_enum: &ExecutableEnum,
    ) -> Vec<syn::Attribute> {
        vec![
            parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug, ::bluejay_typegen::serde::Deserialize)] },
            parse_quote! { #[serde(crate = "bluejay_typegen::serde")] },
            parse_quote! { #[serde(tag = "__typename")] },
        ]
    }

    fn attributes_for_executable_enum_variant(
        &self,
        executable_struct: &ExecutableStruct,
    ) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();

        let serialized_as = syn::LitStr::new(executable_struct.parent_name(), Span::call_site());
        attributes.push(parse_quote! { #[serde(rename = #serialized_as)] });

        if executable_struct.borrows() {
            attributes.push(parse_quote! { #[serde(borrow)] });
        }

        attributes
    }

    fn attributes_for_executable_enum_variant_other(&self) -> Vec<syn::Attribute> {
        vec![parse_quote! { #[serde(other)] }]
    }
}

impl SerdeCodeGenerator {
    fn attributes_for_field(&self, executable_field: &ExecutableField) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();

        let serialized_as = syn::LitStr::new(executable_field.graphql_name(), Span::call_site());
        attributes.push(parse_quote! { #[serde(rename = #serialized_as)] });

        if executable_field.r#type().base().borrows() {
            attributes.push(parse_quote! { #[serde(borrow)] });
        }

        attributes
    }

    fn reference_property_for_type(
        r#type: &WrappedExecutableType,
        property: &syn::Ident,
    ) -> syn::Expr {
        match r#type {
            WrappedExecutableType::Base(_) | WrappedExecutableType::Vec(_) => {
                parse_quote! { &self.#property }
            }
            WrappedExecutableType::Optional(inner) => {
                let inner_reference = Self::reference_property_for_type(inner, property);
                parse_quote! { ::std::option::Option::as_ref(#inner_reference) }
            }
        }
    }
}
