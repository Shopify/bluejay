use crate::executable_definition::ExecutableStruct;
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
    CodeGenerator,
};
use syn::parse_quote;

pub(crate) struct ExecutableEnumVariantBuilder<'a, C: CodeGenerator> {
    executable_struct: &'a ExecutableStruct<'a>,
    code_generator: &'a C,
    /// name of the composite type that contains the field
    composite_type_name: &'a str,
}

impl<'a, C: CodeGenerator> ExecutableEnumVariantBuilder<'a, C> {
    pub(crate) fn build(
        executable_struct: &'a ExecutableStruct<'a>,
        code_generator: &'a C,
        composite_type_name: &'a str,
    ) -> syn::Variant {
        let instance = Self {
            executable_struct,
            code_generator,
            composite_type_name,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let module_ident = instance.module_ident();
        let lifetime = instance.lifetime();

        parse_quote! {
            #(#attributes)*
            #name_ident(#module_ident :: #name_ident #lifetime)
        }
    }

    pub(crate) fn build_other_variant(code_generator: &C) -> syn::Variant {
        let attributes = code_generator.attributes_for_executable_enum_variant_other();
        parse_quote! {
            #(#attributes)*
            Other
        }
    }

    pub(crate) fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_struct.parent_name())
    }

    pub(crate) fn module_ident(&self) -> syn::Ident {
        module_ident(self.composite_type_name)
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_struct.description().map(doc_string));

        attributes.extend(
            self.code_generator
                .attributes_for_executable_enum_variant(self.executable_struct),
        );

        attributes
    }

    fn lifetime(&self) -> Option<syn::Generics> {
        self.executable_struct
            .borrows()
            .then(|| parse_quote! { <'a> })
    }
}
