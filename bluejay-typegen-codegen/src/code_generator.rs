use bluejay_core::definition::{EnumTypeDefinition, EnumValueDefinition};

use crate::{ExecutableEnum, ExecutableField, ExecutableStruct};

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

    /// The body of the field accessor function.
    fn field_accessor_block(
        &self,
        executable_struct: &ExecutableStruct,
        field: &ExecutableField,
    ) -> syn::Block;

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
}
