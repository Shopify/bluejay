use bluejay_core::definition::{
    EnumTypeDefinition, EnumValueDefinition, InputObjectTypeDefinition, InputValueDefinition,
};

use crate::{ExecutableEnum, ExecutableStruct};

pub trait CodeGenerator {
    /// Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_executable_struct(
        &self,
        #[allow(unused_variables)] executable_struct: &ExecutableStruct,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Returns a syn::Fields struct that can be used to generate the fields of an executable struct.
    fn fields_for_executable_struct(&self, executable_struct: &ExecutableStruct) -> syn::Fields;

    /// Any additional impl blocks for the executable struct.
    fn additional_impls_for_executable_struct(
        &self,
        #[allow(unused_variables)] executable_struct: &ExecutableStruct,
    ) -> Vec<syn::ItemImpl> {
        Vec::new()
    }

    /// Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_executable_enum(
        &self,
        #[allow(unused_variables)] executable_enum: &ExecutableEnum,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any additional impl blocks for the executable enum.
    fn additional_impls_for_executable_enum(
        &self,
        #[allow(unused_variables)] executable_enum: &ExecutableEnum,
    ) -> Vec<syn::ItemImpl> {
        Vec::new()
    }

    /// Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_executable_enum_variant(
        &self,
        #[allow(unused_variables)] executable_struct: &ExecutableStruct,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any attributes for the `Other` variant that is added to all enums.
    fn attributes_for_executable_enum_variant_other(&self) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any attributes for the enum type definition. Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_enum(
        &self,
        #[allow(unused_variables)] enum_type_definition: &impl EnumTypeDefinition,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any additional impl blocks for the enum type definition.
    fn additional_impls_for_enum(
        &self,
        #[allow(unused_variables)] enum_type_definition: &impl EnumTypeDefinition,
    ) -> Vec<syn::ItemImpl> {
        Vec::new()
    }

    /// Any attributes for the enum variant. Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_enum_variant(
        &self,
        #[allow(unused_variables)] enum_value_definition: &impl EnumValueDefinition,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any attributes for the `Other` variant that is added to all enums.
    fn attributes_for_enum_variant_other(&self) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any attributes for the input object type definition. Does not need to include the doc string attribute, that will be added automatically.
    /// Note that this does not apply to `@oneOf` input objects, those will use the `attributes_for_one_of_input_object` method instead.
    fn attributes_for_input_object(
        &self,
        #[allow(unused_variables)] input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any additional impl blocks for the input object type definition.
    /// Note that this does not apply to `@oneOf` input objects, those will use the `additional_impls_for_one_of_input_object` method instead.
    fn additional_impls_for_input_object(
        &self,
        #[allow(unused_variables)] input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::ItemImpl> {
        Vec::new()
    }

    /// Any attributes for the input value definition. Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_input_object_field(
        &self,
        #[allow(unused_variables)] input_value_definition: &impl InputValueDefinition,
        #[allow(unused_variables)] borrows: bool,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any attributes for the `@oneOf` input object type definition. Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_one_of_input_object(
        &self,
        #[allow(unused_variables)] input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }

    /// Any additional impl blocks for the `@oneOf` input object type definition.
    fn additional_impls_for_one_of_input_object(
        &self,
        #[allow(unused_variables)] input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::ItemImpl> {
        Vec::new()
    }

    /// Any attributes for the input value definition of a `@oneOf` input object. Does not need to include the doc string attribute, that will be added automatically.
    fn attributes_for_one_of_input_object_field(
        &self,
        #[allow(unused_variables)] input_value_definition: &impl InputValueDefinition,
        #[allow(unused_variables)] borrows: bool,
    ) -> Vec<syn::Attribute> {
        Vec::new()
    }
}
