use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, DefaultContext, EnumTypeDefinition,
    InputObjectTypeDefinition, TypeDefinition,
};
use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    BaseInputTypeReference, InputType as CoreInputType, InputTypeReference,
    SchemaDefinition as CoreSchemaDefinition, ShallowInputTypeReference,
};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug)]
pub struct BaseInputType<'a, C: Context + 'a> {
    name: Name<'a>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> BaseInputType<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn core_type_from_type_definition(
        type_definition: &'a TypeDefinition<'a, C>,
    ) -> Result<BaseInputTypeReference<'a, InputType<'a, C>>, ()> {
        match type_definition {
            TypeDefinition::BuiltinScalar(bstd) => Ok(BaseInputTypeReference::BuiltinScalar(*bstd)),
            TypeDefinition::CustomScalar(cstd) => Ok(BaseInputTypeReference::CustomScalar(cstd)),
            TypeDefinition::Enum(etd) => Ok(BaseInputTypeReference::Enum(etd)),
            TypeDefinition::InputObject(iotd) => Ok(BaseInputTypeReference::InputObject(iotd)),
            TypeDefinition::Interface(_) | TypeDefinition::Object(_) | TypeDefinition::Union(_) => {
                Err(())
            }
        }
    }
}

#[derive(Debug)]
pub enum InputType<'a, C: Context = DefaultContext> {
    Base(BaseInputType<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context> InputType<'a, C> {
    pub(crate) fn base(&self) -> &BaseInputType<'a, C> {
        match self {
            Self::Base(base, _, _) => base,
            Self::List(inner, _, _) => inner.base(),
        }
    }
}

impl<'a, C: Context + 'a> CoreInputType for InputType<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, C>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;

    fn as_ref<
        'b,
        S: CoreSchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
    >(
        &'b self,
        schema_definition: &'b S,
    ) -> InputTypeReference<'b, Self> {
        match self {
            Self::Base(base, required, _) => InputTypeReference::Base(
                schema_definition
                    .get_type_definition(base.name().as_str())
                    .unwrap()
                    .try_into()
                    .unwrap(),
                *required,
            ),
            Self::List(inner, required, _) => {
                InputTypeReference::List(Deref::deref(inner), *required)
            }
        }
    }

    fn as_shallow_ref(&self) -> ShallowInputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required, _) => {
                ShallowInputTypeReference::Base(base.name().as_str(), *required)
            }
            Self::List(inner, required, _) => ShallowInputTypeReference::List(inner, *required),
        }
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for InputType<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens, depth_limiter.bump()?).map(Box::new)?;
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
                context: Default::default(),
            };
            Ok(InputType::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a, C: Context> HasSpan for InputType<'a, C> {
    fn span(&self) -> &Span {
        match self {
            Self::Base(_, _, span) => span,
            Self::List(_, _, span) => span,
        }
    }
}
