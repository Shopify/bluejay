use crate::ast::definition::{Context, Directives, FieldsDefinition, InterfaceImplementations};
use crate::ast::{
    ConstDirectives, DepthLimiter, FromTokens, Parse, ParseError, Tokens, TryFromTokens,
};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::{HasDirectives, ObjectTypeDefinition as CoreObjectTypeDefinition};

#[derive(Debug)]
pub struct ObjectTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: FieldsDefinition<'a, C>,
    is_builtin: bool,
}

impl<'a, C: Context> CoreObjectTypeDefinition for ObjectTypeDefinition<'a, C> {
    type FieldsDefinition = FieldsDefinition<'a, C>;
    type InterfaceImplementations = InterfaceImplementations<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn interface_implementations(&self) -> Option<&Self::InterfaceImplementations> {
        self.interface_implementations.as_ref()
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        &self.fields_definition
    }

    fn is_builtin(&self) -> bool {
        self.is_builtin
    }
}

impl<'a, C: Context> ObjectTypeDefinition<'a, C> {
    pub(crate) const TYPE_IDENTIFIER: &'static str = "type";
    const __SCHEMA_DEFINITION: &'static str = "type __Schema {
        description: String
        types: [__Type!]!
        queryType: __Type!
        mutationType: __Type
        subscriptionType: __Type
        directives: [__Directive!]!
    }";
    const __TYPE_DEFINITION: &'static str = "type __Type {
        kind: __TypeKind!
        name: String
        description: String
        # must be non-null for OBJECT and INTERFACE, otherwise null.
        fields(includeDeprecated: Boolean = false): [__Field!]
        # must be non-null for OBJECT and INTERFACE, otherwise null.
        interfaces: [__Type!]
        # must be non-null for INTERFACE and UNION, otherwise null.
        possibleTypes: [__Type!]
        # must be non-null for ENUM, otherwise null.
        enumValues(includeDeprecated: Boolean = false): [__EnumValue!]
        # must be non-null for INPUT_OBJECT, otherwise null.
        inputFields(includeDeprecated: Boolean = false): [__InputValue!]
        # must be non-null for NON_NULL and LIST, otherwise null.
        ofType: __Type
        # may be non-null for custom SCALAR, otherwise null.
        specifiedByURL: String
    }";
    const __FIELD_DEFINITION: &'static str = "type __Field {
        name: String!
        description: String
        args(includeDeprecated: Boolean = false): [__InputValue!]!
        type: __Type!
        isDeprecated: Boolean!
        deprecationReason: String
    }";
    const __INPUT_VALUE_DEFINITION: &'static str = "type __InputValue {
        name: String!
        description: String
        type: __Type!
        defaultValue: String
    }";
    const __ENUM_VALUE_DEFINITION: &'static str = "type __EnumValue {
        name: String!
        description: String
        isDeprecated: Boolean!
        deprecationReason: String
    }";
    const __DIRECTIVE_DEFINITION: &'static str = "type __Directive {
        name: String!
        description: String
        locations: [__DirectiveLocation!]!
        args(includeDeprecated: Boolean = false): [__InputValue!]!
        isRepeatable: Boolean!
    }";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    fn builtin(s: &'static str) -> Self {
        let mut definition = Self::parse(s).unwrap();
        definition.is_builtin = true;
        definition
    }

    pub(crate) fn __schema() -> Self {
        Self::builtin(Self::__SCHEMA_DEFINITION)
    }

    pub(crate) fn __type() -> Self {
        Self::builtin(Self::__TYPE_DEFINITION)
    }

    pub(crate) fn __field() -> Self {
        Self::builtin(Self::__FIELD_DEFINITION)
    }

    pub(crate) fn __input_value() -> Self {
        Self::builtin(Self::__INPUT_VALUE_DEFINITION)
    }

    pub(crate) fn __enum_value() -> Self {
        Self::builtin(Self::__ENUM_VALUE_DEFINITION)
    }

    pub(crate) fn __directive() -> Self {
        Self::builtin(Self::__DIRECTIVE_DEFINITION)
    }

    pub(crate) fn add_query_root_fields(&mut self) {
        self.fields_definition.add_query_root_fields();
    }
}

impl<'a, C: Context> FromTokens<'a> for ObjectTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::TYPE_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let directives =
            ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let fields_definition = FieldsDefinition::from_tokens(tokens, depth_limiter.bump()?)?;
        Ok(Self {
            description,
            name,
            interface_implementations,
            directives: directives.map(Directives::from),
            fields_definition,
            is_builtin: false,
        })
    }
}

impl<'a, C: Context> HasDirectives for ObjectTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
