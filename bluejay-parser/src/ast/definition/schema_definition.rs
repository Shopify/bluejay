use crate::ast::definition::{
    ArgumentsDefinition, Context, CustomScalarTypeDefinition, DefaultContext, Directive,
    DirectiveDefinition, Directives, EnumTypeDefinition, EnumValueDefinition, EnumValueDefinitions,
    FieldDefinition, FieldsDefinition, InputFieldsDefinition, InputObjectTypeDefinition, InputType,
    InputValueDefinition, InterfaceImplementation, InterfaceImplementations,
    InterfaceTypeDefinition, ObjectTypeDefinition, OutputType, TypeDefinition, UnionMemberType,
    UnionMemberTypes, UnionTypeDefinition,
};
use crate::lexical_token::StringValue;
use bluejay_core::definition::{
    HasDirectives, InterfaceImplementation as CoreInterfaceImplementation,
    ObjectTypeDefinition as CoreObjectTypeDefinition, SchemaDefinition as CoreSchemaDefinition,
    TypeDefinition as CoreTypeDefinition, TypeDefinitionReference,
};
use bluejay_core::AsIter;
use std::collections::{btree_map::Values, BTreeMap, HashMap};

#[derive(Debug)]
pub struct SchemaDefinition<'a, C: Context = DefaultContext> {
    type_definitions: BTreeMap<&'a str, &'a TypeDefinition<'a, C>>,
    directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a, C>>,
    description: Option<&'a StringValue<'a>>,
    query: &'a ObjectTypeDefinition<'a, C>,
    mutation: Option<&'a ObjectTypeDefinition<'a, C>>,
    subscription: Option<&'a ObjectTypeDefinition<'a, C>>,
    directives: Option<&'a Directives<'a, C>>,
    interface_implementors: HashMap<&'a str, Vec<&'a ObjectTypeDefinition<'a, C>>>,
}

impl<'a, C: Context> SchemaDefinition<'a, C> {
    pub(crate) fn new(
        type_definitions: BTreeMap<&'a str, &'a TypeDefinition<'a, C>>,
        directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a, C>>,
        description: Option<&'a StringValue>,
        query: &'a ObjectTypeDefinition<'a, C>,
        mutation: Option<&'a ObjectTypeDefinition<'a, C>>,
        subscription: Option<&'a ObjectTypeDefinition<'a, C>>,
        directives: Option<&'a Directives<'a, C>>,
    ) -> Self {
        let interface_implementors = Self::interface_implementors(&type_definitions);
        Self {
            type_definitions,
            directive_definitions,
            description,
            query,
            mutation,
            subscription,
            directives,
            interface_implementors,
        }
    }

    fn interface_implementors(
        type_definitions: &BTreeMap<&'a str, &'a TypeDefinition<'a, C>>,
    ) -> HashMap<&'a str, Vec<&'a ObjectTypeDefinition<'a, C>>> {
        type_definitions.values().fold(
            HashMap::new(),
            |mut interface_implementors, &type_definition| {
                if let TypeDefinition::Object(otd) = type_definition {
                    if let Some(interface_implementations) = otd.interface_implementations() {
                        interface_implementations
                            .iter()
                            .for_each(|interface_implementation| {
                                let itd_name = interface_implementation.name();
                                interface_implementors
                                    .entry(itd_name)
                                    .or_default()
                                    .push(otd);
                            });
                    }
                }

                interface_implementors
            },
        )
    }
}

impl<'a, C: Context> CoreSchemaDefinition for SchemaDefinition<'a, C> {
    type Directive = Directive<'a, C>;
    type Directives = Directives<'a, C>;
    type InputValueDefinition = InputValueDefinition<'a, C>;
    type InputFieldsDefinition = InputFieldsDefinition<'a, C>;
    type ArgumentsDefinition = ArgumentsDefinition<'a, C>;
    type EnumValueDefinition = EnumValueDefinition<'a, C>;
    type EnumValueDefinitions = EnumValueDefinitions<'a, C>;
    type FieldDefinition = FieldDefinition<'a, C>;
    type FieldsDefinition = FieldsDefinition<'a, C>;
    type InterfaceImplementation = InterfaceImplementation<'a, C>;
    type InterfaceImplementations = InterfaceImplementations<'a, C>;
    type UnionMemberType = UnionMemberType<'a, C>;
    type UnionMemberTypes = UnionMemberTypes<'a, C>;
    type InputType = InputType<'a, C>;
    type OutputType = OutputType<'a, C>;
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, C>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, C>;
    type TypeDefinition = TypeDefinition<'a, C>;
    type DirectiveDefinition = DirectiveDefinition<'a, C>;
    type TypeDefinitions<'b>
        = std::iter::Map<
        Values<'b, &'b str, &'b TypeDefinition<'a, C>>,
        fn(&&'b TypeDefinition<'a, C>) -> TypeDefinitionReference<'b, TypeDefinition<'a, C>>,
    >
    where
        'a: 'b;
    type DirectiveDefinitions<'b>
        = std::iter::Copied<Values<'b, &'b str, &'b DirectiveDefinition<'a, C>>>
    where
        'a: 'b;
    type InterfaceImplementors<'b>
        = std::iter::Flatten<
        std::option::IntoIter<
            std::iter::Copied<std::slice::Iter<'b, &'b ObjectTypeDefinition<'a, C>>>,
        >,
    >
    where
        'a: 'b;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn query(&self) -> &Self::ObjectTypeDefinition {
        self.query
    }

    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.mutation
    }

    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.subscription
    }

    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<TypeDefinitionReference<'_, Self::TypeDefinition>> {
        self.type_definitions.get(name).map(|td| td.as_ref())
    }

    fn type_definitions(&self) -> Self::TypeDefinitions<'_> {
        self.type_definitions.values().map(
            |td: &&TypeDefinition<C>| -> TypeDefinitionReference<'_, TypeDefinition<'a, C>> {
                td.as_ref()
            },
        )
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        self.directive_definitions.get(name).copied()
    }

    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_> {
        self.directive_definitions.values().copied()
    }

    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_> {
        self.interface_implementors
            .get(itd.name().as_ref())
            .map(|implementors| implementors.iter().copied())
            .into_iter()
            .flatten()
    }
}

impl<'a, C: Context> HasDirectives for SchemaDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives
    }
}
