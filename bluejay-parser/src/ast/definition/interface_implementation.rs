use crate::ast::definition::InterfaceTypeDefinition;
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::Name;
use bluejay_core::definition::InterfaceImplementation as CoreInterfaceImplementation;
use once_cell::unsync::OnceCell;

#[derive(Debug)]
pub struct InterfaceImplementation<'a> {
    name: Name<'a>,
    r#type: OnceCell<&'a InterfaceTypeDefinition<'a>>,
}

impl<'a> CoreInterfaceImplementation for InterfaceImplementation<'a> {
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a>;

    fn interface(&self) -> &Self::InterfaceTypeDefinition {
        self.r#type.get().unwrap()
    }
}

impl<'a> InterfaceImplementation<'a> {
    pub(crate) fn set_type_reference(
        &self,
        type_reference: &'a InterfaceTypeDefinition<'a>,
    ) -> Result<(), &'a InterfaceTypeDefinition<'a>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn interface_name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a> FromTokens<'a> for InterfaceImplementation<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().map(|name| Self {
            name,
            r#type: OnceCell::new(),
        })
    }
}
