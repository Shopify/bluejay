use crate::ast::definition::{
    CustomScalarTypeDefinition, EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    TypeDefinitionReference, UnionTypeDefinition,
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

#[derive(Debug)]
pub struct BaseOutputTypeReference<'a> {
    name: Name<'a>,
    r#type: OnceCell<BaseOutputTypeReferenceFromAbstract<'a, Self>>,
}

impl<'a> AbstractBaseOutputTypeReference for BaseOutputTypeReference<'a> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a>;
    type UnionTypeDefinition = UnionTypeDefinition<'a>;

    fn as_ref(&self) -> BaseOutputTypeReferenceFromAbstract<'_, Self> {
        *self.r#type.get().unwrap()
    }
}

impl<'a> BaseOutputTypeReference<'a> {
    fn new(name: Name<'a>) -> Self {
        Self {
            name,
            r#type: OnceCell::new(),
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
        type_definition_reference: &'a TypeDefinitionReference<'a>,
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
pub enum OutputTypeReference<'a> {
    Base(BaseOutputTypeReference<'a>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a> AbstractOutputTypeReference for OutputTypeReference<'a> {
    type BaseOutputTypeReference = BaseOutputTypeReference<'a>;

    fn as_ref(&self) -> OutputTypeReferenceFromAbstract<'_, Self> {
        match self {
            Self::Base(base, required, _) => CoreOutputTypeReference::Base(base, *required),
            Self::List(inner, required, _) => {
                CoreOutputTypeReference::List(inner.as_ref(), *required)
            }
        }
    }
}

impl<'a> OutputTypeReference<'a> {
    pub(crate) fn non_null_string() -> Self {
        Self::Base(
            BaseOutputTypeReference::new(Name::new("String", Span::empty())),
            true,
            Span::empty(),
        )
    }
}

impl<'a> FromTokens<'a> for OutputTypeReference<'a> {
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
