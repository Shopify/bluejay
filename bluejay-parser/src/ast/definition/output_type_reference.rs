use crate::ast::definition::{
    CustomScalarTypeDefinition, EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    TypeDefinitionReference, UnionTypeDefinition,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{HasSpan, Name, PunctuatorType};
use crate::Span;
use bluejay_core::definition::{
    AbstractBaseOutputTypeReference, AbstractOutputTypeReference,
    BaseOutputTypeReference as CoreBaseOutputTypeReference, BaseOutputTypeReferenceFromAbstract,
    OutputTypeReference as CoreOutputTypeReference,
};
use once_cell::unsync::OnceCell;

#[derive(Debug)]
pub struct BaseOutputTypeReference<'a> {
    name: Name<'a>,
    r#type: OnceCell<BaseOutputTypeReferenceFromAbstract<Self>>,
}

impl<'a> AbstractBaseOutputTypeReference for BaseOutputTypeReference<'a> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a>;
    type UnionTypeDefinition = UnionTypeDefinition<'a>;
    type WrappedCustomScalarTypeDefinition = &'a CustomScalarTypeDefinition<'a>;
    type WrappedEnumTypeDefinition = &'a EnumTypeDefinition<'a>;
    type WrappedInterfaceTypeDefinition = &'a InterfaceTypeDefinition<'a>;
    type WrappedObjectTypeDefinition = &'a ObjectTypeDefinition<'a>;
    type WrappedUnionTypeDefinition = &'a UnionTypeDefinition<'a>;
}

impl<'a> AsRef<BaseOutputTypeReferenceFromAbstract<Self>> for BaseOutputTypeReference<'a> {
    fn as_ref(&self) -> &BaseOutputTypeReferenceFromAbstract<Self> {
        self.r#type.get().unwrap()
    }
}

impl<'a> BaseOutputTypeReference<'a> {
    fn new(name: Name<'a>) -> Self {
        Self {
            name,
            r#type: Default::default(),
        }
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: BaseOutputTypeReferenceFromAbstract<Self>,
    ) -> Result<(), BaseOutputTypeReferenceFromAbstract<Self>> {
        self.r#type.set(type_reference)
    }

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn core_type_from_type_definition_reference(
        type_definition_reference: &'a TypeDefinitionReference<'a>,
    ) -> Result<BaseOutputTypeReferenceFromAbstract<Self>, ()> {
        match type_definition_reference {
            TypeDefinitionReference::BuiltinScalarType(bstd) => {
                Ok(CoreBaseOutputTypeReference::BuiltinScalarType(*bstd))
            }
            TypeDefinitionReference::CustomScalarType(cstd, pd) => {
                Ok(CoreBaseOutputTypeReference::CustomScalarType(cstd, *pd))
            }
            TypeDefinitionReference::EnumType(etd, pd) => {
                Ok(CoreBaseOutputTypeReference::EnumType(etd, *pd))
            }
            TypeDefinitionReference::InputObjectType(_, _) => Err(()),
            TypeDefinitionReference::InterfaceType(itd, pd) => {
                Ok(CoreBaseOutputTypeReference::InterfaceType(itd, *pd))
            }
            TypeDefinitionReference::ObjectType(otd, pd) => {
                Ok(CoreBaseOutputTypeReference::ObjectType(otd, *pd))
            }
            TypeDefinitionReference::UnionType(utd, pd) => {
                Ok(CoreBaseOutputTypeReference::UnionType(utd, *pd))
            }
        }
    }
}

#[derive(Debug)]
pub struct OutputTypeReference<'a> {
    inner: CoreOutputTypeReference<BaseOutputTypeReference<'a>, Box<Self>>,
    _span: Span,
}

impl<'a> AbstractOutputTypeReference for OutputTypeReference<'a> {
    type BaseOutputTypeReference = BaseOutputTypeReference<'a>;
    type Wrapper = Box<Self>;
}

impl<'a> AsRef<CoreOutputTypeReference<BaseOutputTypeReference<'a>, Box<Self>>>
    for OutputTypeReference<'a>
{
    fn as_ref(&self) -> &CoreOutputTypeReference<BaseOutputTypeReference<'a>, Box<Self>> {
        &self.inner
    }
}

impl<'a> AsRef<CoreOutputTypeReference<BaseOutputTypeReference<'a>, Box<OutputTypeReference<'a>>>>
    for Box<OutputTypeReference<'a>>
{
    fn as_ref(
        &self,
    ) -> &CoreOutputTypeReference<BaseOutputTypeReference<'a>, Box<OutputTypeReference<'a>>> {
        let inner: &OutputTypeReference<'a> = self.as_ref();
        inner.as_ref()
    }
}

impl<'a> OutputTypeReference<'a> {
    pub(crate) fn non_null_string() -> OutputTypeReference<'a> {
        OutputTypeReference {
            inner: CoreOutputTypeReference::Base(
                BaseOutputTypeReference::new(Name::new("String", Span::empty())),
                true,
            ),
            _span: Span::empty(),
        }
    }

    pub(crate) fn base(&self) -> &BaseOutputTypeReference<'a> {
        match &self.inner {
            CoreOutputTypeReference::Base(b, _) => b,
            CoreOutputTypeReference::List(inner, _) => inner.base(),
        }
    }
}

impl<'a> FromTokens<'a> for OutputTypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner_self = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            let inner = CoreOutputTypeReference::List(inner_self, bang_span.is_some());
            Ok(Self { inner, _span: span })
        } else if let Some(base_name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = if let Some(bang_span) = &bang_span {
                base_name.span().merge(bang_span)
            } else {
                base_name.span().clone()
            };
            let base = BaseOutputTypeReference::new(base_name);
            let inner = CoreOutputTypeReference::Base(base, bang_span.is_some());
            Ok(Self { inner, _span: span })
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
