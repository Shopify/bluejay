use crate::ast::definition::{Context, Directives, EnumValueDefinitions};
use crate::ast::{ConstDirectives, FromTokens, Parse, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::{EnumTypeDefinition as CoreEnumTypeDefinition, HasDirectives};

#[derive(Debug)]
pub struct EnumTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    enum_value_definitions: EnumValueDefinitions<'a, C>,
    is_builtin: bool,
}

impl<'a, C: Context> CoreEnumTypeDefinition for EnumTypeDefinition<'a, C> {
    type EnumValueDefinitions = EnumValueDefinitions<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions {
        &self.enum_value_definitions
    }

    fn is_builtin(&self) -> bool {
        self.is_builtin
    }
}

impl<'a, C: Context> EnumTypeDefinition<'a, C> {
    pub(crate) const ENUM_IDENTIFIER: &'static str = "enum";
    const __TYPE_KIND_DEFINITION: &'static str = "enum __TypeKind {
        SCALAR
        OBJECT
        INTERFACE
        UNION
        ENUM
        INPUT_OBJECT
        LIST
        NON_NULL
    }";
    const __DIRECTIVE_LOCATION_DEFINITION: &'static str = "enum __DirectiveLocation {
        QUERY
        MUTATION
        SUBSCRIPTION
        FIELD
        FRAGMENT_DEFINITION
        FRAGMENT_SPREAD
        INLINE_FRAGMENT
        VARIABLE_DEFINITION
        SCHEMA
        SCALAR
        OBJECT
        FIELD_DEFINITION
        ARGUMENT_DEFINITION
        INTERFACE
        UNION
        ENUM
        ENUM_VALUE
        INPUT_OBJECT
        INPUT_FIELD_DEFINITION
    }";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    fn builtin(s: &'static str) -> Self {
        let mut definition = Self::parse(s).unwrap();
        definition.is_builtin = true;
        definition
    }

    pub(crate) fn __type_kind() -> Self {
        Self::builtin(Self::__TYPE_KIND_DEFINITION)
    }

    pub(crate) fn __directive_location() -> Self {
        Self::builtin(Self::__DIRECTIVE_LOCATION_DEFINITION)
    }
}

impl<'a, C: Context> FromTokens<'a> for EnumTypeDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::ENUM_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        let enum_value_definitions = EnumValueDefinitions::from_tokens(tokens)?;
        Ok(Self {
            description,
            name,
            directives: directives.map(Directives::from),
            enum_value_definitions,
            is_builtin: false,
        })
    }
}

impl<'a, C: Context> AsRef<EnumTypeDefinition<'a, C>> for EnumTypeDefinition<'a, C> {
    fn as_ref(&self) -> &EnumTypeDefinition<'a, C> {
        self
    }
}

impl<'a, C: Context> HasDirectives for EnumTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
