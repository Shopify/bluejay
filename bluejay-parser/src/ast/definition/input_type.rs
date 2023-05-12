use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, DefaultContext, EnumTypeDefinition,
    InputObjectTypeDefinition, TypeDefinitionReference,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    BaseInputType as CoreBaseInputType, BaseInputTypeReference, InputType as CoreInputType,
    InputTypeReference,
};
use once_cell::sync::OnceCell;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BaseInputType<'a, C: Context + 'a> {
    name: Name<'a>,
    r#type: OnceCell<BaseInputTypeReference<'a, Self>>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> CoreBaseInputType for BaseInputType<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;

    fn as_ref(&self) -> BaseInputTypeReference<'a, Self> {
        *self.r#type.get().unwrap()
    }
}

impl<'a, C: Context + 'a> BaseInputType<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: BaseInputTypeReference<'a, Self>,
    ) -> Result<(), BaseInputTypeReference<'a, Self>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn core_type_from_type_definition_reference(
        type_definition_reference: &'a TypeDefinitionReference<'a, C>,
    ) -> Result<BaseInputTypeReference<'a, Self>, ()> {
        match type_definition_reference {
            TypeDefinitionReference::BuiltinScalar(bstd) => {
                Ok(BaseInputTypeReference::BuiltinScalar(*bstd))
            }
            TypeDefinitionReference::CustomScalar(cstd) => {
                Ok(BaseInputTypeReference::CustomScalar(cstd))
            }
            TypeDefinitionReference::Enum(etd) => Ok(BaseInputTypeReference::Enum(etd)),
            TypeDefinitionReference::InputObject(iotd) => {
                Ok(BaseInputTypeReference::InputObject(iotd))
            }
            TypeDefinitionReference::Interface(_)
            | TypeDefinitionReference::Object(_)
            | TypeDefinitionReference::Union(_) => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum InputType<'a, C: Context = DefaultContext> {
    Base(BaseInputType<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context + 'a> CoreInputType for InputType<'a, C> {
    type BaseInputType = BaseInputType<'a, C>;

    fn as_ref(&self) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required, _) => InputTypeReference::Base(base, *required),
            Self::List(inner, required, _) => InputTypeReference::List(inner.as_ref(), *required),
        }
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for InputType<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            Ok(InputType::List(inner, bang_span.is_some(), span))
        } else if let Some(base_name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = if let Some(bang_span) = &bang_span {
                base_name.span().merge(bang_span)
            } else {
                base_name.span().clone()
            };
            let base = BaseInputType {
                name: base_name,
                r#type: OnceCell::new(),
                context: Default::default(),
            };
            Ok(InputType::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
