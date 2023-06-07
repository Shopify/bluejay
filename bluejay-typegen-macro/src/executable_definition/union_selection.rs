use crate::attributes::doc_string;
use crate::names::module_ident;
use crate::{
    executable_definition::{fields_and_definitions, named_fields, nested_module, Context},
    names::{enum_variant_ident, type_ident},
};
use bluejay_core::definition::{
    ObjectTypeDefinition, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};
use bluejay_core::executable::{InlineFragment, Selection, SelectionReference, SelectionSet};
use bluejay_core::AsIter;
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) fn generate_union_type_definition(
    union_type_definition: &impl UnionTypeDefinition,
    selection_set: &impl SelectionSet,
    context: Context,
) -> Vec<syn::Item> {
    let description = union_type_definition.description().map(doc_string);

    let ident = type_ident(context.name());

    let inline_fragments_and_definitions =
        inline_fragments_and_definitions(selection_set, union_type_definition);

    // when fragment spread is used
    if inline_fragments_and_definitions.is_empty() {
        return Vec::new();
    }

    let is_not_exhaustive =
        inline_fragments_and_definitions.len() != union_type_definition.union_member_types().len();

    let fields_and_definitions: Vec<_> = inline_fragments_and_definitions
        .iter()
        .map(|(inline_fragment, otd)| {
            fields_and_definitions(inline_fragment.selection_set(), otd.fields_definition())
        })
        .collect();

    let variants = inline_fragments_and_definitions
        .iter()
        .zip(&fields_and_definitions)
        .map(|((_, otd), fields_and_definitions)| {
            let description = otd.description().map(doc_string);
            let ident = enum_variant_ident(otd.name());
            let rename = syn::LitStr::new(otd.name(), Span::call_site());

            let fields = named_fields(fields_and_definitions, &context, false);

            parse_quote! {
                #description
                #[serde(rename = #rename)]
                #ident #fields
            }
        })
        .chain(is_not_exhaustive.then(|| {
            parse_quote! {
                #[serde(other)]
                Other
            }
        }))
        .collect::<Vec<syn::Variant>>();

    let nested = inline_fragments_and_definitions
        .iter()
        .zip(&fields_and_definitions)
        .filter_map(|((_, otd), fields_and_definitions)| {
            nested_module(fields_and_definitions, &context.dive(otd.name()))
        })
        .collect::<Vec<syn::Item>>();

    let lifetime: Option<syn::Generics> = fields_and_definitions
        .iter()
        .any(|fields_and_definitions| {
            context.selection_set_contains_reference_types(
                fields_and_definitions,
                &mut Default::default(),
            )
        })
        .then(|| parse_quote! { <'a> });

    let mut items = vec![parse_quote! {
        #description
        #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug, ::bluejay_typegen::serde::Deserialize)]
        #[serde(crate = "bluejay_typegen::serde")]
        #[serde(tag = "__typename")]
        pub enum #ident #lifetime { #(#variants),* }
    }];

    if !nested.is_empty() {
        let module_ident = module_ident(context.name());
        items.push(parse_quote! {
            pub mod #module_ident {
                #(#nested)*
            }
        });
    }

    items
}

type ObjectTypeDefinitionForUnionTypeDefinition<U> = <<<U as UnionTypeDefinition>::UnionMemberTypes as UnionMemberTypes>::UnionMemberType as UnionMemberType>::ObjectTypeDefinition;

pub(crate) fn inline_fragments_and_definitions<'a, S: SelectionSet, U: UnionTypeDefinition>(
    selection_set: &'a S,
    union_type_definition: &'a U,
) -> Vec<(
    &'a <S::Selection as Selection>::InlineFragment,
    &'a ObjectTypeDefinitionForUnionTypeDefinition<U>,
)> {
    selection_set
        .iter()
        .filter_map(|selection| match selection.as_ref() {
            SelectionReference::InlineFragment(i) => Some((
                i,
                union_type_definition
                    .union_member_types()
                    .get(i.type_condition().unwrap())
                    .unwrap()
                    .member_type(),
            )),
            SelectionReference::Field(_) | SelectionReference::FragmentSpread(_) => None,
        })
        .collect()
}
