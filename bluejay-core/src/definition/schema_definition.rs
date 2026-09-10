use crate::definition::{
    ArgumentsDefinition, Directive, DirectiveDefinition, Directives, EnumTypeDefinition,
    EnumValueDefinition, EnumValueDefinitions, FieldDefinition, FieldsDefinition, HasDirectives,
    InputFieldsDefinition, InputObjectTypeDefinition, InputType, InputValueDefinition,
    InterfaceImplementation, InterfaceImplementations, InterfaceTypeDefinition,
    ObjectTypeDefinition, OutputType, ScalarTypeDefinition, TypeDefinition,
    TypeDefinitionReference, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};

pub trait SchemaDefinition:
    Sized + HasDirectives<Directives = <Self as SchemaDefinition>::Directives>
{
    type Directive: Directive<SchemaDefinition = Self>;
    type Directives: Directives<SchemaDefinition = Self>;
    type InputValueDefinition: InputValueDefinition<SchemaDefinition = Self>;
    type InputFieldsDefinition: InputFieldsDefinition<SchemaDefinition = Self>;
    type ArgumentsDefinition: ArgumentsDefinition<SchemaDefinition = Self>;
    type EnumValueDefinition: EnumValueDefinition<SchemaDefinition = Self>;
    type EnumValueDefinitions: EnumValueDefinitions<SchemaDefinition = Self>;
    type FieldDefinition: FieldDefinition<SchemaDefinition = Self>;
    type FieldsDefinition: FieldsDefinition<SchemaDefinition = Self>;
    type InterfaceImplementation: InterfaceImplementation<SchemaDefinition = Self>;
    type InterfaceImplementations: InterfaceImplementations<SchemaDefinition = Self>;
    type UnionMemberType: UnionMemberType<SchemaDefinition = Self>;
    type UnionMemberTypes: UnionMemberTypes<SchemaDefinition = Self>;
    type InputType: InputType<SchemaDefinition = Self>;
    type OutputType: OutputType<SchemaDefinition = Self>;
    type CustomScalarTypeDefinition: ScalarTypeDefinition<SchemaDefinition = Self>;
    type ObjectTypeDefinition: ObjectTypeDefinition<SchemaDefinition = Self>;
    type InterfaceTypeDefinition: InterfaceTypeDefinition<SchemaDefinition = Self>;
    type UnionTypeDefinition: UnionTypeDefinition<SchemaDefinition = Self>;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<SchemaDefinition = Self>;
    type EnumTypeDefinition: EnumTypeDefinition<SchemaDefinition = Self>;
    type TypeDefinition: TypeDefinition<SchemaDefinition = Self>;
    type DirectiveDefinition: DirectiveDefinition<SchemaDefinition = Self>;
    type TypeDefinitions<'a>: Iterator<Item = TypeDefinitionReference<'a, Self>>
    where
        Self: 'a;
    type DirectiveDefinitions<'a>: Iterator<Item = &'a Self::DirectiveDefinition>
    where
        Self: 'a;
    type InterfaceImplementors<'a>: Iterator<Item = &'a Self::ObjectTypeDefinition>
    where
        Self: 'a;

    fn description(&self) -> Option<&str>;
    fn query(&self) -> &Self::ObjectTypeDefinition;
    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn get_type_definition(&self, name: &str) -> Option<TypeDefinitionReference<'_, Self>>;
    fn type_definitions(&self) -> Self::TypeDefinitions<'_>;
    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition>;
    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_>;
    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_>;
}
