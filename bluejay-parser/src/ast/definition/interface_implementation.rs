use crate::ast::definition::{Context, InterfaceTypeDefinition};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::Name;
use bluejay_core::definition::InterfaceImplementation as CoreInterfaceImplementation;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct InterfaceImplementation<'a, C: Context> {
    name: Name<'a>,
    r#type: OnceCell<&'a InterfaceTypeDefinition<'a, C>>,
}

impl<'a, C: Context> CoreInterfaceImplementation for InterfaceImplementation<'a, C> {
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;

    fn interface(&self) -> &Self::InterfaceTypeDefinition {
        self.r#type.get().unwrap()
    }
}

impl<'a, C: Context> InterfaceImplementation<'a, C> {
    pub(crate) fn set_type_reference(
        &self,
        type_reference: &'a InterfaceTypeDefinition<'a, C>,
    ) -> Result<(), &'a InterfaceTypeDefinition<'a, C>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn interface_name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for InterfaceImplementation<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().map(|name| Self {
            name,
            r#type: OnceCell::new(),
        })
    }
}
