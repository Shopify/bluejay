use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, EnumTypeDefinition, InterfaceTypeDefinition,
    ObjectTypeDefinition, TypeDefinition, UnionTypeDefinition,
};
use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    BaseOutputTypeReference, OutputType as CoreOutputType, OutputTypeReference,
    SchemaDefinition as CoreSchemaDefinition, ShallowOutputTypeReference,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BaseOutputType<'a, C: Context + 'a> {
    name: Name<'a>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> BaseOutputType<'a, C> {
    fn new(name: Name<'a>) -> Self {
        Self {
            name,
            context: Default::default(),
        }
    }

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn core_type_from_type_definition(
        type_definition: &'a TypeDefinition<'a, C>,
    ) -> Result<BaseOutputTypeReference<'a, OutputType<'a, C>>, ()> {
        match type_definition {
            TypeDefinition::BuiltinScalar(bstd) => {
                Ok(BaseOutputTypeReference::BuiltinScalar(*bstd))
            }
            TypeDefinition::CustomScalar(cstd) => Ok(BaseOutputTypeReference::CustomScalar(cstd)),
            TypeDefinition::Enum(etd) => Ok(BaseOutputTypeReference::Enum(etd)),
            TypeDefinition::InputObject(_) => Err(()),
            TypeDefinition::Interface(itd) => Ok(BaseOutputTypeReference::Interface(itd)),
            TypeDefinition::Object(otd) => Ok(BaseOutputTypeReference::Object(otd)),
            TypeDefinition::Union(utd) => Ok(BaseOutputTypeReference::Union(utd)),
        }
    }
}

#[derive(Debug)]
pub enum OutputType<'a, C: Context + 'a> {
    Base(BaseOutputType<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context + 'a> OutputType<'a, C> {
    pub(crate) fn base(&self) -> &BaseOutputType<'a, C> {
        match self {
            Self::Base(base, _, _) => base,
            Self::List(inner, _, _) => inner.base(),
        }
    }
}

impl<'a, C: Context + 'a> CoreOutputType for OutputType<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, C>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, C>;

    fn as_ref<
        'b,
        S: CoreSchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
            ObjectTypeDefinition = Self::ObjectTypeDefinition,
            InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
            UnionTypeDefinition = Self::UnionTypeDefinition,
        >,
    >(
        &'b self,
        schema_definition: &'b S,
    ) -> OutputTypeReference<'b, Self> {
        match self {
            Self::Base(base, required, _) => OutputTypeReference::Base(
                schema_definition
                    .get_type_definition(base.name().as_str())
                    .unwrap()
                    .try_into()
                    .unwrap(),
                *required,
            ),
            Self::List(inner, required, _) => OutputTypeReference::List(inner.as_ref(), *required),
        }
    }

    fn as_shallow_ref(&self) -> ShallowOutputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required, _) => {
                ShallowOutputTypeReference::Base(base.name().as_str(), *required)
            }
            Self::List(inner, required, _) => {
                ShallowOutputTypeReference::List(inner.as_ref(), *required)
            }
        }
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for OutputType<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens, depth_limiter.bump()?).map(Box::new)?;
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
            let base = BaseOutputType::new(base_name);
            Ok(Self::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
