use crate::ast::definition::{
    Context, Directives, EnumValueDefinitions, FieldsDefinition, InputFieldsDefinition,
    InterfaceImplementations, RootOperationTypeDefinition, UnionMemberTypes,
};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType};
use bluejay_core::definition::HasDirectives;

const POSSIBLE_EXTENSION_IDENTIFIERS: &[&str] = &[
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
const ENUM_EXTENSION_COMPONENTS: &[&str] = &["@", "{"];
const INPUT_OBJECT_EXTENSION_COMPONENTS: &[&str] = &["@", "{"];

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

impl<C: Context> TypeSystemExtension<'_, C> {
    pub(crate) const EXTEND_IDENTIFIER: &'static str = "extend";
}

impl<'a, C: Context> TypeSystemExtension<'a, C> {
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
}

impl<'a, C: Context> FromTokens<'a> for TypeSystemExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::EXTEND_IDENTIFIER)?;

        match tokens.peek_name(0).map(AsRef::as_ref) {
            Some(SchemaExtension::<C>::SCHEMA_IDENTIFIER) => Ok(Self::Schema(
                SchemaExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(ScalarTypeExtension::<C>::SCALAR_IDENTIFIER) => Ok(Self::Scalar(
                ScalarTypeExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(ObjectTypeExtension::<C>::TYPE_IDENTIFIER) => Ok(Self::Object(
                ObjectTypeExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(InterfaceTypeExtension::<C>::INTERFACE_IDENTIFIER) => Ok(Self::Interface(
                InterfaceTypeExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(UnionTypeExtension::<C>::UNION_IDENTIFIER) => Ok(Self::Union(
                UnionTypeExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(EnumTypeExtension::<C>::ENUM_IDENTIFIER) => Ok(Self::Enum(
                EnumTypeExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(InputObjectTypeExtension::<C>::INPUT_IDENTIFIER) => Ok(Self::InputObject(
                InputObjectTypeExtension::from_tokens(tokens, depth_limiter.bump()?)?,
            )),
            Some(_) => Err(ParseError::ExpectedOneOf {
                span: tokens.expect_name()?.into(),
                values: POSSIBLE_EXTENSION_IDENTIFIERS,
            }),
            None => Err(tokens.unexpected_token()),
        }
    }
}

#[derive(Debug)]
pub struct SchemaExtension<'a, C: Context> {
    directives: Option<Directives<'a, C>>,
    root_operation_type_definitions: Option<Vec<RootOperationTypeDefinition<'a>>>,
}

impl<C: Context> SchemaExtension<'_, C> {
    pub(crate) const SCHEMA_IDENTIFIER: &'static str = "schema";
}

impl<'a, C: Context> SchemaExtension<'a, C> {
    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn root_operation_type_definitions(&self) -> Option<&[RootOperationTypeDefinition<'a>]> {
        self.root_operation_type_definitions.as_deref()
    }
}

impl<'a, C: Context> FromTokens<'a> for SchemaExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::SCHEMA_IDENTIFIER)?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let root_operation_type_definitions =
            if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
                Some(parse_root_operation_type_definitions(
                    tokens,
                    depth_limiter.bump()?,
                )?)
            } else {
                None
            };

        expect_extension_component(
            directives.is_some() || root_operation_type_definitions.is_some(),
            tokens,
            SCHEMA_EXTENSION_COMPONENTS,
        )?;

        Ok(Self {
            directives: directives.map(Directives::from),
            root_operation_type_definitions,
        })
    }
}

#[derive(Debug)]
pub struct ScalarTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Directives<'a, C>,
}

impl<C: Context> ScalarTypeExtension<'_, C> {
    pub(crate) const SCALAR_IDENTIFIER: &'static str = "scalar";
}

impl<'a, C: Context> ScalarTypeExtension<'a, C> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn directives(&self) -> &Directives<'a, C> {
        &self.directives
    }
}

impl<'a, C: Context> FromTokens<'a> for ScalarTypeExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::SCALAR_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        expect_extension_component(directives.is_some(), tokens, SCALAR_EXTENSION_COMPONENTS)?;
        let directives = directives.expect("directives were present");

        Ok(Self {
            name,
            directives: Directives::from(directives),
        })
    }
}

#[derive(Debug)]
pub struct ObjectTypeExtension<'a, C: Context> {
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: Option<FieldsDefinition<'a, C>>,
}

impl<C: Context> ObjectTypeExtension<'_, C> {
    pub(crate) const TYPE_IDENTIFIER: &'static str = "type";
}

impl<'a, C: Context> ObjectTypeExtension<'a, C> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn interface_implementations(&self) -> Option<&InterfaceImplementations<'a, C>> {
        self.interface_implementations.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn fields_definition(&self) -> Option<&FieldsDefinition<'a, C>> {
        self.fields_definition.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for ObjectTypeExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::TYPE_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let fields_definition = parse_optional_fields_definition(tokens, depth_limiter.bump()?)?;

        expect_extension_component(
            interface_implementations.is_some()
                || directives.is_some()
                || fields_definition.is_some(),
            tokens,
            OBJECT_OR_INTERFACE_EXTENSION_COMPONENTS,
        )?;

        Ok(Self {
            name,
            interface_implementations,
            directives: directives.map(Directives::from),
            fields_definition,
        })
    }
}

#[derive(Debug)]
pub struct InterfaceTypeExtension<'a, C: Context> {
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: Option<FieldsDefinition<'a, C>>,
}

impl<C: Context> InterfaceTypeExtension<'_, C> {
    pub(crate) const INTERFACE_IDENTIFIER: &'static str = "interface";
}

impl<'a, C: Context> InterfaceTypeExtension<'a, C> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn interface_implementations(&self) -> Option<&InterfaceImplementations<'a, C>> {
        self.interface_implementations.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn fields_definition(&self) -> Option<&FieldsDefinition<'a, C>> {
        self.fields_definition.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for InterfaceTypeExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::INTERFACE_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let fields_definition = parse_optional_fields_definition(tokens, depth_limiter.bump()?)?;

        expect_extension_component(
            interface_implementations.is_some()
                || directives.is_some()
                || fields_definition.is_some(),
            tokens,
            OBJECT_OR_INTERFACE_EXTENSION_COMPONENTS,
        )?;

        Ok(Self {
            name,
            interface_implementations,
            directives: directives.map(Directives::from),
            fields_definition,
        })
    }
}

#[derive(Debug)]
pub struct UnionTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    member_types: Option<UnionMemberTypes<'a, C>>,
}

impl<C: Context> UnionTypeExtension<'_, C> {
    pub(crate) const UNION_IDENTIFIER: &'static str = "union";
}

impl<'a, C: Context> UnionTypeExtension<'a, C> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn union_member_types(&self) -> Option<&UnionMemberTypes<'a, C>> {
        self.member_types.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for UnionTypeExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::UNION_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let member_types = if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
            Some(UnionMemberTypes::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?)
        } else {
            None
        };

        expect_extension_component(
            directives.is_some() || member_types.is_some(),
            tokens,
            UNION_EXTENSION_COMPONENTS,
        )?;

        Ok(Self {
            name,
            directives: directives.map(Directives::from),
            member_types,
        })
    }
}

#[derive(Debug)]
pub struct EnumTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    enum_value_definitions: Option<EnumValueDefinitions<'a, C>>,
}

impl<C: Context> EnumTypeExtension<'_, C> {
    pub(crate) const ENUM_IDENTIFIER: &'static str = "enum";
}

impl<'a, C: Context> EnumTypeExtension<'a, C> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn enum_value_definitions(&self) -> Option<&EnumValueDefinitions<'a, C>> {
        self.enum_value_definitions.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for EnumTypeExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
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

        expect_extension_component(
            directives.is_some() || enum_value_definitions.is_some(),
            tokens,
            ENUM_EXTENSION_COMPONENTS,
        )?;

        Ok(Self {
            name,
            directives: directives.map(Directives::from),
            enum_value_definitions,
        })
    }
}

#[derive(Debug)]
pub struct InputObjectTypeExtension<'a, C: Context> {
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    input_fields_definition: Option<InputFieldsDefinition<'a, C>>,
}

impl<C: Context> InputObjectTypeExtension<'_, C> {
    pub(crate) const INPUT_IDENTIFIER: &'static str = "input";
}

impl<'a, C: Context> InputObjectTypeExtension<'a, C> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub fn input_field_definitions(&self) -> Option<&InputFieldsDefinition<'a, C>> {
        self.input_fields_definition.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for InputObjectTypeExtension<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
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

        expect_extension_component(
            directives.is_some() || input_fields_definition.is_some(),
            tokens,
            INPUT_OBJECT_EXTENSION_COMPONENTS,
        )?;

        Ok(Self {
            name,
            directives: directives.map(Directives::from),
            input_fields_definition,
        })
    }
}

impl<'a, C: Context> HasDirectives for TypeSystemExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        TypeSystemExtension::directives(self)
    }
}

impl<'a, C: Context> HasDirectives for SchemaExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl<'a, C: Context> HasDirectives for ScalarTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        Some(&self.directives)
    }
}

impl<'a, C: Context> HasDirectives for ObjectTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl<'a, C: Context> HasDirectives for InterfaceTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl<'a, C: Context> HasDirectives for UnionTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl<'a, C: Context> HasDirectives for EnumTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl<'a, C: Context> HasDirectives for InputObjectTypeExtension<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

fn parse_root_operation_type_definitions<'a>(
    tokens: &mut impl Tokens<'a>,
    depth_limiter: DepthLimiter,
) -> Result<Vec<RootOperationTypeDefinition<'a>>, ParseError> {
    tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
    let mut root_operation_type_definitions = Vec::with_capacity(3);
    loop {
        root_operation_type_definitions.push(RootOperationTypeDefinition::from_tokens(
            tokens,
            depth_limiter.bump()?,
        )?);
        if tokens
            .next_if_punctuator(PunctuatorType::CloseBrace)
            .is_some()
        {
            break;
        }
    }
    Ok(root_operation_type_definitions)
}

fn parse_optional_fields_definition<'a, C: Context>(
    tokens: &mut impl Tokens<'a>,
    depth_limiter: DepthLimiter,
) -> Result<Option<FieldsDefinition<'a, C>>, ParseError> {
    if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
        Ok(Some(FieldsDefinition::from_tokens_without_builtin_fields(
            tokens,
            depth_limiter,
        )?))
    } else {
        Ok(None)
    }
}

fn expect_extension_component<'a>(
    found_component: bool,
    tokens: &mut impl Tokens<'a>,
    expected_components: &'static [&'static str],
) -> Result<(), ParseError> {
    if found_component {
        Ok(())
    } else if let Some(span) = tokens.peek_span(0) {
        Err(ParseError::ExpectedOneOf {
            span,
            values: expected_components,
        })
    } else {
        Err(tokens.unexpected_eof())
    }
}
