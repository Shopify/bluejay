use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, DefaultContext, EnumTypeDefinition,
    InputObjectTypeDefinition, TypeDefinitionReference,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    AbstractBaseInputTypeReference, AbstractInputTypeReference,
    BaseInputTypeReference as CoreBaseInputTypeReference, BaseInputTypeReferenceFromAbstract,
    InputTypeReference as CoreInputTypeReference, InputTypeReferenceFromAbstract,
};
use once_cell::sync::OnceCell;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BaseInputTypeReference<'a, C: Context + 'a> {
    name: Name<'a>,
    r#type: OnceCell<BaseInputTypeReferenceFromAbstract<'a, Self>>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> AbstractBaseInputTypeReference for BaseInputTypeReference<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;

    fn as_ref(&self) -> BaseInputTypeReferenceFromAbstract<'a, Self> {
        *self.r#type.get().unwrap()
    }
}

impl<'a, C: Context + 'a> BaseInputTypeReference<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: BaseInputTypeReferenceFromAbstract<'a, Self>,
    ) -> Result<(), BaseInputTypeReferenceFromAbstract<'a, Self>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn core_type_from_type_definition_reference(
        type_definition_reference: &'a TypeDefinitionReference<'a, C>,
    ) -> Result<BaseInputTypeReferenceFromAbstract<'a, Self>, ()> {
        match type_definition_reference {
            TypeDefinitionReference::BuiltinScalar(bstd) => {
                Ok(CoreBaseInputTypeReference::BuiltinScalarType(*bstd))
            }
            TypeDefinitionReference::CustomScalar(cstd) => {
                Ok(CoreBaseInputTypeReference::CustomScalarType(cstd))
            }
            TypeDefinitionReference::Enum(etd) => Ok(CoreBaseInputTypeReference::EnumType(etd)),
            TypeDefinitionReference::InputObject(iotd) => {
                Ok(CoreBaseInputTypeReference::InputObjectType(iotd))
            }
            TypeDefinitionReference::Interface(_)
            | TypeDefinitionReference::Object(_)
            | TypeDefinitionReference::Union(_) => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum InputTypeReference<'a, C: Context = DefaultContext> {
    Base(BaseInputTypeReference<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context + 'a> AbstractInputTypeReference for InputTypeReference<'a, C> {
    type BaseInputTypeReference = BaseInputTypeReference<'a, C>;

    fn as_ref(&self) -> InputTypeReferenceFromAbstract<'_, Self> {
        match self {
            Self::Base(base, required, _) => CoreInputTypeReference::Base(base, *required),
            Self::List(inner, required, _) => {
                CoreInputTypeReference::List(inner.as_ref(), *required)
            }
        }
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for InputTypeReference<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            Ok(InputTypeReference::List(inner, bang_span.is_some(), span))
        } else if let Some(base_name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = if let Some(bang_span) = &bang_span {
                base_name.span().merge(bang_span)
            } else {
                base_name.span().clone()
            };
            let base = BaseInputTypeReference {
                name: base_name,
                r#type: OnceCell::new(),
                context: Default::default(),
            };
            Ok(InputTypeReference::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
