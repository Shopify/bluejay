//! The parser keeps each extension as its own node, in source order, so that tools can read
//! extensions as they are written. Bluejay does not apply extensions to the definitions they
//! extend: building a [`SchemaDefinition`](crate::ast::definition::SchemaDefinition) from a
//! document that contains extensions is an error.

use crate::ast::definition::{
    Context, Directives, EnumValueDefinitions, FieldsDefinition, InputFieldsDefinition,
    InterfaceImplementations, RootOperationTypeDefinition, UnionMemberTypes,
};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::HasDirectives;

const EXTENSION_KINDS: &[&str] = &[
    "schema",
    "scalar",
    "type",
    "interface",
    "union",
    "enum",
    "input",
];
const SCHEMA_EXTENSION_COMPONENTS: &[&str] = &["@", "{"];
const SCALAR_EXTENSION_COMPONENTS: &[&str] = &["@"];
const OBJECT_OR_INTERFACE_EXTENSION_COMPONENTS: &[&str] = &["implements", "@", "{"];
const UNION_EXTENSION_COMPONENTS: &[&str] = &["@", "="];
const ENUM_OR_INPUT_OBJECT_EXTENSION_COMPONENTS: &[&str] = &["@", "{"];

#[derive(Debug)]
pub enum TypeSystemExtension<'a, C: Context> {
    Schema(SchemaExtension<'a, C>),
    Scalar(ScalarTypeExtension<'a, C>),
    Object(ObjectTypeExtension<'a, C>),
    Interface(InterfaceTypeExtension<'a, C>),
    Union(UnionTypeExtension<'a, C>),
    Enum(EnumTypeExtension<'a, C>),
    InputObject(InputObjectTypeExtension<'a, C>),
}

impl<'a, C: Context> TypeSystemExtension<'a, C> {
    pub(crate) const EXTEND_IDENTIFIER: &'static str = "extend";

    pub fn name_token(&self) -> Option<&Name<'a>> {
        match self {
            Self::Schema(_) => None,
            Self::Scalar(extension) => Some(extension.name_token()),
            Self::Object(extension) => Some(extension.name_token()),
            Self::Interface(extension) => Some(extension.name_token()),
            Self::Union(extension) => Some(extension.name_token()),
            Self::Enum(extension) => Some(extension.name_token()),
            Self::InputObject(extension) => Some(extension.name_token()),
        }
    }

    pub fn name(&self) -> Option<&'a str> {
        self.name_token().map(Name::as_str)
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        match self {
            Self::Schema(extension) => extension.directives(),
            Self::Scalar(extension) => Some(extension.directives()),
            Self::Object(extension) => extension.directives(),
            Self::Interface(extension) => extension.directives(),
            Self::Union(extension) => extension.directives(),
            Self::Enum(extension) => extension.directives(),
            Self::InputObject(extension) => extension.directives(),
        }
    }

    /// From `extend` to the type name (or `schema`), so that diagnostics do not mark the whole
    /// extension, which can be long.
    pub(crate) fn header_span(&self) -> Span {
        let end = match self {
            Self::Schema(extension) => &extension.schema_identifier_span,
            Self::Scalar(extension) => extension.name.span(),
            Self::Object(extension) => extension.name.span(),
            Self::Interface(extension) => extension.name.span(),
            Self::Union(extension) => extension.name.span(),
            Self::Enum(extension) => extension.name.span(),
            Self::InputObject(extension) => extension.name.span(),
        };
        Span::new(self.span().byte_range().start..end.byte_range().end)
    }
}

impl<'a, C: Context> FromTokens<'a> for TypeSystemExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        if let Some(description) = tokens.next_if_string_value() {
            return Err(ParseError::TypeSystemExtensionWithDescription {
                span: *description.span(),
            });
        }
        let extend_span = tokens.expect_name_value(Self::EXTEND_IDENTIFIER)?;
        let depth_limiter = depth_limiter.bump()?;

        match tokens.peek_name(0).map(Name::as_str) {
            Some(SchemaExtension::<C>::SCHEMA_IDENTIFIER) => {
                SchemaExtension::from_tokens(tokens, depth_limiter, extend_span).map(Self::Schema)
            }
            Some(ScalarTypeExtension::<C>::SCALAR_IDENTIFIER) => {
                ScalarTypeExtension::from_tokens(tokens, depth_limiter, extend_span)
                    .map(Self::Scalar)
            }
            Some(ObjectTypeExtension::<C>::TYPE_IDENTIFIER) => {
                ObjectTypeExtension::from_tokens(tokens, depth_limiter, extend_span)
                    .map(Self::Object)
            }
            Some(InterfaceTypeExtension::<C>::INTERFACE_IDENTIFIER) => {
                InterfaceTypeExtension::from_tokens(tokens, depth_limiter, extend_span)
                    .map(Self::Interface)
            }
            Some(UnionTypeExtension::<C>::UNION_IDENTIFIER) => {
                UnionTypeExtension::from_tokens(tokens, depth_limiter, extend_span).map(Self::Union)
            }
            Some(EnumTypeExtension::<C>::ENUM_IDENTIFIER) => {
                EnumTypeExtension::from_tokens(tokens, depth_limiter, extend_span).map(Self::Enum)
            }
            Some(InputObjectTypeExtension::<C>::INPUT_IDENTIFIER) => {
                InputObjectTypeExtension::from_tokens(tokens, depth_limiter, extend_span)
                    .map(Self::InputObject)
            }
            _ => Err(expected_one_of(tokens, EXTENSION_KINDS)),
        }
    }
}

impl<C: Context> HasSpan for TypeSystemExtension<'_, C> {
    fn span(&self) -> &Span {
        match self {
            Self::Schema(extension) => extension.span(),
            Self::Scalar(extension) => extension.span(),
            Self::Object(extension) => extension.span(),
            Self::Interface(extension) => extension.span(),
            Self::Union(extension) => extension.span(),
            Self::Enum(extension) => extension.span(),
            Self::InputObject(extension) => extension.span(),
        }
    }
}

impl<'a, C: Context> HasDirectives for TypeSystemExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        TypeSystemExtension::directives(self)
    }
}

/// `extend schema Directives[Const]? { RootOperationTypeDefinition+ }` or
/// `extend schema Directives[Const]`.
#[derive(Debug)]
pub struct SchemaExtension<'a, C: Context> {
    schema_identifier_span: Span,
    directives: Option<Directives<'a, C>>,
    root_operation_type_definitions: Option<Vec<RootOperationTypeDefinition<'a>>>,
    span: Span,
}

impl<'a, C: Context> SchemaExtension<'a, C> {
    pub(crate) const SCHEMA_IDENTIFIER: &'static str = "schema";

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn root_operation_type_definitions(&self) -> Option<&[RootOperationTypeDefinition<'a>]> {
        self.root_operation_type_definitions.as_deref()
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        let schema_identifier_span = tokens.expect_name_value(Self::SCHEMA_IDENTIFIER)?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let root_operation_type_definitions =
            if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
                Some(RootOperationTypeDefinition::list_from_tokens(
                    tokens,
                    depth_limiter,
                )?)
            } else {
                None
            };
        let end = root_operation_type_definitions
            .as_ref()
            .map(|(_, span)| *span)
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .ok_or(ParseError::EmptyTypeSystemExtension {
                span: extend_span.merge(&schema_identifier_span),
                expected: SCHEMA_EXTENSION_COMPONENTS,
            })?;
        Ok(Self {
            schema_identifier_span,
            directives: directives.map(Directives::from),
            root_operation_type_definitions: root_operation_type_definitions
                .map(|(root_operation_type_definitions, _)| root_operation_type_definitions),
            span: extend_span.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for SchemaExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for SchemaExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

/// `extend scalar Name Directives[Const]`.
#[derive(Debug)]
pub struct ScalarTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Directives<'a, C>,
    span: Span,
}

impl<'a, C: Context> ScalarTypeExtension<'a, C> {
    pub(crate) const SCALAR_IDENTIFIER: &'static str = "scalar";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name(&self) -> &'a str {
        self.name.as_str()
    }

    pub fn directives(&self) -> &Directives<'a, C> {
        &self.directives
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::SCALAR_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?.ok_or(
            ParseError::EmptyTypeSystemExtension {
                span: extend_span.merge(name.span()),
                expected: SCALAR_EXTENSION_COMPONENTS,
            },
        )?;
        // A parsed directive list is never empty, so it always has a span.
        let end = directives.span().copied().unwrap_or(*name.span());
        Ok(Self {
            name,
            directives: Directives::from(directives),
            span: extend_span.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for ScalarTypeExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for ScalarTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        Some(&self.directives)
    }
}

struct FieldsTypeExtensionParts<'a, C: Context> {
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: Option<FieldsDefinition<'a, C>>,
    span: Span,
}

impl<'a, C: Context> FieldsTypeExtensionParts<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let fields_definition = if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
            Some(FieldsDefinition::extension_from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?)
        } else {
            None
        };
        let end = fields_definition
            .as_ref()
            .map(|fields_definition| *fields_definition.span())
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .or_else(|| {
                interface_implementations
                    .as_ref()
                    .and_then(InterfaceImplementations::span)
            })
            .ok_or(ParseError::EmptyTypeSystemExtension {
                span: extend_span.merge(name.span()),
                expected: OBJECT_OR_INTERFACE_EXTENSION_COMPONENTS,
            })?;
        Ok(Self {
            name,
            interface_implementations,
            directives: directives.map(Directives::from),
            fields_definition,
            span: extend_span.merge(&end),
        })
    }
}

/// `extend type Name ImplementsInterfaces? Directives[Const]? FieldsDefinition?`, with at least
/// one of the optional parts.
#[derive(Debug)]
pub struct ObjectTypeExtension<'a, C: Context> {
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: Option<FieldsDefinition<'a, C>>,
    span: Span,
}

impl<'a, C: Context> ObjectTypeExtension<'a, C> {
    pub(crate) const TYPE_IDENTIFIER: &'static str = "type";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name(&self) -> &'a str {
        self.name.as_str()
    }

    pub fn interface_implementations(&self) -> Option<&InterfaceImplementations<'a, C>> {
        self.interface_implementations.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    /// Unlike the fields of a type definition, these do not include the builtin `__typename`.
    pub fn fields_definition(&self) -> Option<&FieldsDefinition<'a, C>> {
        self.fields_definition.as_ref()
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::TYPE_IDENTIFIER)?;
        let FieldsTypeExtensionParts {
            name,
            interface_implementations,
            directives,
            fields_definition,
            span,
        } = FieldsTypeExtensionParts::from_tokens(tokens, depth_limiter, extend_span)?;
        Ok(Self {
            name,
            interface_implementations,
            directives,
            fields_definition,
            span,
        })
    }
}

impl<C: Context> HasSpan for ObjectTypeExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for ObjectTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

/// `extend interface Name ImplementsInterfaces? Directives[Const]? FieldsDefinition?`, with at
/// least one of the optional parts.
#[derive(Debug)]
pub struct InterfaceTypeExtension<'a, C: Context> {
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: Option<FieldsDefinition<'a, C>>,
    span: Span,
}

impl<'a, C: Context> InterfaceTypeExtension<'a, C> {
    pub(crate) const INTERFACE_IDENTIFIER: &'static str = "interface";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name(&self) -> &'a str {
        self.name.as_str()
    }

    pub fn interface_implementations(&self) -> Option<&InterfaceImplementations<'a, C>> {
        self.interface_implementations.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    /// Unlike the fields of a type definition, these do not include the builtin `__typename`.
    pub fn fields_definition(&self) -> Option<&FieldsDefinition<'a, C>> {
        self.fields_definition.as_ref()
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::INTERFACE_IDENTIFIER)?;
        let FieldsTypeExtensionParts {
            name,
            interface_implementations,
            directives,
            fields_definition,
            span,
        } = FieldsTypeExtensionParts::from_tokens(tokens, depth_limiter, extend_span)?;
        Ok(Self {
            name,
            interface_implementations,
            directives,
            fields_definition,
            span,
        })
    }
}

impl<C: Context> HasSpan for InterfaceTypeExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for InterfaceTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

/// `extend union Name Directives[Const]? UnionMemberTypes` or
/// `extend union Name Directives[Const]`.
#[derive(Debug)]
pub struct UnionTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    union_member_types: Option<UnionMemberTypes<'a, C>>,
    span: Span,
}

impl<'a, C: Context> UnionTypeExtension<'a, C> {
    pub(crate) const UNION_IDENTIFIER: &'static str = "union";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name(&self) -> &'a str {
        self.name.as_str()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn union_member_types(&self) -> Option<&UnionMemberTypes<'a, C>> {
        self.union_member_types.as_ref()
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::UNION_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let union_member_types = if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
            Some(UnionMemberTypes::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?)
        } else {
            None
        };
        let end = union_member_types
            .as_ref()
            .and_then(UnionMemberTypes::span)
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .ok_or(ParseError::EmptyTypeSystemExtension {
                span: extend_span.merge(name.span()),
                expected: UNION_EXTENSION_COMPONENTS,
            })?;
        Ok(Self {
            name,
            directives: directives.map(Directives::from),
            union_member_types,
            span: extend_span.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for UnionTypeExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for UnionTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

/// `extend enum Name Directives[Const]? EnumValuesDefinition` or
/// `extend enum Name Directives[Const]`.
#[derive(Debug)]
pub struct EnumTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    enum_value_definitions: Option<EnumValueDefinitions<'a, C>>,
    span: Span,
}

impl<'a, C: Context> EnumTypeExtension<'a, C> {
    pub(crate) const ENUM_IDENTIFIER: &'static str = "enum";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name(&self) -> &'a str {
        self.name.as_str()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn enum_value_definitions(&self) -> Option<&EnumValueDefinitions<'a, C>> {
        self.enum_value_definitions.as_ref()
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::ENUM_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let enum_value_definitions = if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace)
        {
            Some(EnumValueDefinitions::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?)
        } else {
            None
        };
        let end = enum_value_definitions
            .as_ref()
            .map(|enum_value_definitions| *enum_value_definitions.span())
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .ok_or(ParseError::EmptyTypeSystemExtension {
                span: extend_span.merge(name.span()),
                expected: ENUM_OR_INPUT_OBJECT_EXTENSION_COMPONENTS,
            })?;
        Ok(Self {
            name,
            directives: directives.map(Directives::from),
            enum_value_definitions,
            span: extend_span.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for EnumTypeExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for EnumTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

/// `extend input Name Directives[Const]? InputFieldsDefinition` or
/// `extend input Name Directives[Const]`.
#[derive(Debug)]
pub struct InputObjectTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    input_fields_definition: Option<InputFieldsDefinition<'a, C>>,
    span: Span,
}

impl<'a, C: Context> InputObjectTypeExtension<'a, C> {
    pub(crate) const INPUT_IDENTIFIER: &'static str = "input";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name(&self) -> &'a str {
        self.name.as_str()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn input_field_definitions(&self) -> Option<&InputFieldsDefinition<'a, C>> {
        self.input_fields_definition.as_ref()
    }

    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
        extend_span: Span,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::INPUT_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let input_fields_definition =
            if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
                Some(InputFieldsDefinition::from_tokens(
                    tokens,
                    depth_limiter.bump()?,
                )?)
            } else {
                None
            };
        let end = input_fields_definition
            .as_ref()
            .map(|input_fields_definition| *input_fields_definition.span())
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .ok_or(ParseError::EmptyTypeSystemExtension {
                span: extend_span.merge(name.span()),
                expected: ENUM_OR_INPUT_OBJECT_EXTENSION_COMPONENTS,
            })?;
        Ok(Self {
            name,
            directives: directives.map(Directives::from),
            input_fields_definition,
            span: extend_span.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for InputObjectTypeExtension<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for InputObjectTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

/// Does not consume the token, so that the document parser can continue at it: it is often the
/// start of the next definition.
fn expected_one_of<'a>(
    tokens: &mut impl Tokens<'a>,
    values: &'static [&'static str],
) -> ParseError {
    match tokens.peek_span(0) {
        Some(span) => ParseError::ExpectedOneOf { span, values },
        None => tokens.unexpected_eof(),
    }
}
