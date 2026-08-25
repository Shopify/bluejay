use std::marker::PhantomData;

use crate::ast::definition::{Context, InterfaceTypeDefinition, SchemaDefinition};
use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::Name;
use bluejay_core::definition::{
    InterfaceImplementation as CoreInterfaceImplementation,
    SchemaDefinition as CoreSchemaDefinition,
};

#[derive(Debug)]
pub struct InterfaceImplementation<'a, C: Context + 'a> {
    name: Name<'a>,
    context: PhantomData<C>,
}

impl<'a, C: Context> CoreInterfaceImplementation for InterfaceImplementation<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn interface<'b>(
        &'b self,
        schema_definition: &'b SchemaDefinition<'a, C>,
    ) -> &'b InterfaceTypeDefinition<'a, C> {
        schema_definition
            .get_type_definition(self.interface_name().as_str())
            .unwrap()
            .as_interface()
            .unwrap()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<'a, C: Context> InterfaceImplementation<'a, C> {
    pub(crate) fn interface_name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for InterfaceImplementation<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>, _: DepthLimiter) -> Result<Self, ParseError> {
        tokens.expect_name().map(|name| Self {
            name,
            context: PhantomData,
        })
    }
}
