use crate::ast::definition::{
    CustomScalarTypeDefinition, EnumTypeDefinition, InputObjectTypeDefinition,
    TypeDefinitionReference,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    AbstractBaseInputTypeReference, AbstractInputTypeReference,
    BaseInputTypeReference as CoreBaseInputTypeReference, BaseInputTypeReferenceFromAbstract,
    InputTypeReference as CoreInputTypeReference,
};
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct BaseInputTypeReference<'a> {
    name: Name<'a>,
    r#type: OnceCell<BaseInputTypeReferenceFromAbstract<Self>>,
}

impl<'a> AbstractBaseInputTypeReference for BaseInputTypeReference<'a> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a>;
    type WrappedCustomScalarTypeDefinition = &'a CustomScalarTypeDefinition<'a>;
    type WrappedEnumTypeDefinition = &'a EnumTypeDefinition<'a>;
    type WrappedInputObjectTypeDefinition = &'a InputObjectTypeDefinition<'a>;
}

impl<'a> AsRef<BaseInputTypeReferenceFromAbstract<Self>> for BaseInputTypeReference<'a> {
    fn as_ref(&self) -> &BaseInputTypeReferenceFromAbstract<Self> {
        self.r#type.get().unwrap()
    }
}

impl<'a> BaseInputTypeReference<'a> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: BaseInputTypeReferenceFromAbstract<Self>,
    ) -> Result<(), BaseInputTypeReferenceFromAbstract<Self>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn core_type_from_type_definition_reference(
        type_definition_reference: &'a TypeDefinitionReference<'a>,
    ) -> Result<BaseInputTypeReferenceFromAbstract<Self>, ()> {
        match type_definition_reference {
            TypeDefinitionReference::BuiltinScalarType(bstd) => {
                Ok(CoreBaseInputTypeReference::BuiltinScalarType(*bstd))
            }
            TypeDefinitionReference::CustomScalarType(cstd, pd) => {
                Ok(CoreBaseInputTypeReference::CustomScalarType(cstd, *pd))
            }
            TypeDefinitionReference::EnumType(etd, pd) => {
                Ok(CoreBaseInputTypeReference::EnumType(etd, *pd))
            }
            TypeDefinitionReference::InputObjectType(iotd, pd) => {
                Ok(CoreBaseInputTypeReference::InputObjectType(iotd, *pd))
            }
            TypeDefinitionReference::InterfaceType(_, _)
            | TypeDefinitionReference::ObjectType(_, _)
            | TypeDefinitionReference::UnionType(_, _) => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct InputTypeReference<'a> {
    inner: CoreInputTypeReference<BaseInputTypeReference<'a>, Box<Self>>,
    _span: Span,
}

impl<'a> AbstractInputTypeReference for InputTypeReference<'a> {
    type BaseInputTypeReference = BaseInputTypeReference<'a>;
    type Wrapper = Box<Self>;
}

impl<'a> AsRef<CoreInputTypeReference<BaseInputTypeReference<'a>, Box<Self>>>
    for InputTypeReference<'a>
{
    fn as_ref(&self) -> &CoreInputTypeReference<BaseInputTypeReference<'a>, Box<Self>> {
        &self.inner
    }
}

impl<'a> AsRef<CoreInputTypeReference<BaseInputTypeReference<'a>, Box<InputTypeReference<'a>>>>
    for Box<InputTypeReference<'a>>
{
    fn as_ref(
        &self,
    ) -> &CoreInputTypeReference<BaseInputTypeReference<'a>, Box<InputTypeReference<'a>>> {
        let inner: &InputTypeReference<'a> = self.as_ref();
        inner.as_ref()
    }
}

impl<'a> InputTypeReference<'a> {
    pub(crate) fn base(&self) -> &BaseInputTypeReference<'a> {
        match &self.inner {
            CoreInputTypeReference::Base(b, _) => b,
            CoreInputTypeReference::List(inner, _) => inner.base(),
        }
    }
}

impl<'a> FromTokens<'a> for InputTypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner_self = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            let inner = CoreInputTypeReference::List(inner_self, bang_span.is_some());
            Ok(Self { inner, _span: span })
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
            };
            let inner = CoreInputTypeReference::Base(base, bang_span.is_some());
            Ok(Self { inner, _span: span })
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
