use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, EnumTypeDefinition, InterfaceTypeDefinition,
    ObjectTypeDefinition, TypeDefinitionReference, UnionTypeDefinition,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    AbstractBaseOutputTypeReference, AbstractOutputTypeReference,
    BaseOutputTypeReference as CoreBaseOutputTypeReference, BaseOutputTypeReferenceFromAbstract,
    OutputTypeReference as CoreOutputTypeReference, OutputTypeReferenceFromAbstract,
};
use once_cell::sync::OnceCell;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BaseOutputTypeReference<'a, C: Context + 'a> {
    name: Name<'a>,
    r#type: OnceCell<BaseOutputTypeReferenceFromAbstract<'a, Self>>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> AbstractBaseOutputTypeReference for BaseOutputTypeReference<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, C>;

    fn as_ref(&self) -> BaseOutputTypeReferenceFromAbstract<'_, Self> {
        *self.r#type.get().unwrap()
    }
}

impl<'a, C: Context + 'a> BaseOutputTypeReference<'a, C> {
    fn new(name: Name<'a>) -> Self {
        Self {
            name,
            r#type: OnceCell::new(),
            context: Default::default(),
        }
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: BaseOutputTypeReferenceFromAbstract<'a, Self>,
    ) -> Result<(), BaseOutputTypeReferenceFromAbstract<'a, Self>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn core_type_from_type_definition_reference(
        type_definition_reference: &'a TypeDefinitionReference<'a, C>,
    ) -> Result<BaseOutputTypeReferenceFromAbstract<'a, Self>, ()> {
        match type_definition_reference {
            TypeDefinitionReference::BuiltinScalar(bstd) => {
                Ok(CoreBaseOutputTypeReference::BuiltinScalarType(*bstd))
            }
            TypeDefinitionReference::CustomScalar(cstd) => {
                Ok(CoreBaseOutputTypeReference::CustomScalarType(cstd))
            }
            TypeDefinitionReference::Enum(etd) => Ok(CoreBaseOutputTypeReference::EnumType(etd)),
            TypeDefinitionReference::InputObject(_) => Err(()),
            TypeDefinitionReference::Interface(itd) => {
                Ok(CoreBaseOutputTypeReference::InterfaceType(itd))
            }
            TypeDefinitionReference::Object(otd) => {
                Ok(CoreBaseOutputTypeReference::ObjectType(otd))
            }
            TypeDefinitionReference::Union(utd) => Ok(CoreBaseOutputTypeReference::UnionType(utd)),
        }
    }
}

#[derive(Debug)]
pub enum OutputTypeReference<'a, C: Context + 'a> {
    Base(BaseOutputTypeReference<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context + 'a> AbstractOutputTypeReference for OutputTypeReference<'a, C> {
    type BaseOutputTypeReference = BaseOutputTypeReference<'a, C>;

    fn as_ref(&self) -> OutputTypeReferenceFromAbstract<'_, Self> {
        match self {
            Self::Base(base, required, _) => CoreOutputTypeReference::Base(base, *required),
            Self::List(inner, required, _) => {
                CoreOutputTypeReference::List(inner.as_ref(), *required)
            }
        }
    }
}

impl<'a, C: Context + 'a> OutputTypeReference<'a, C> {
    pub(crate) fn non_null_string() -> Self {
        Self::Base(
            BaseOutputTypeReference::new(Name::new("String", Span::empty())),
            true,
            Span::empty(),
        )
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for OutputTypeReference<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            Ok(Self::List(inner, bang_span.is_some(), span))
        } else if let Some(base_name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = if let Some(bang_span) = &bang_span {
                base_name.span().merge(bang_span)
            } else {
                base_name.span().clone()
            };
            let base = BaseOutputTypeReference::new(base_name);
            Ok(Self::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
