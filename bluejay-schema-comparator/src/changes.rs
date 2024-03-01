use crate::diff::ArgumentForDirective;
use bluejay_core::definition::{
    prelude::*, DirectiveDefinition, DirectiveLocation, EnumTypeDefinition, EnumValueDefinition,
    FieldDefinition, HasDirectives, InputObjectTypeDefinition, InputType, InputValueDefinition,
    ObjectTypeDefinition, OutputType, SchemaDefinition, ShallowInputTypeReference,
    ShallowOutputTypeReference, TypeDefinitionReference, UnionTypeDefinition,
};
use bluejay_core::{Argument, AsIter, Directive, Value};
use bluejay_printer::value::ValuePrinter;
use std::borrow::Cow;
use strum::AsRefStr;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum Criticality {
    Breaking { reason: Cow<'static, str> },
    Dangerous { reason: Cow<'static, str> },
    Safe { reason: Cow<'static, str> },
}

impl Criticality {
    fn breaking(reason: Option<Cow<'static, str>>) -> Self {
        Self::Breaking {
            reason: reason.unwrap_or(Cow::from("This change is a breaking change")),
        }
    }

    fn dangerous(reason: Option<Cow<'static, str>>) -> Self {
        Self::Dangerous {
            reason: reason.unwrap_or(Cow::from("This change is dangerous")),
        }
    }

    fn safe(reason: Option<Cow<'static, str>>) -> Self {
        Self::Safe {
            reason: reason.unwrap_or(Cow::from("This change is safe")),
        }
    }

    pub fn is_breaking(&self) -> bool {
        matches!(self, Self::Breaking { .. })
    }

    pub fn is_dangerous(&self) -> bool {
        matches!(self, Self::Dangerous { .. })
    }

    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Safe { .. })
    }

    pub fn reason(&self) -> &str {
        match self {
            Self::Breaking { reason } => reason.as_ref(),
            Self::Dangerous { reason } => reason.as_ref(),
            Self::Safe { reason } => reason.as_ref(),
        }
    }
}

#[derive(AsRefStr)]
pub enum Change<'a, S: SchemaDefinition> {
    TypeRemoved {
        removed_type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
    },
    TypeAdded {
        added_type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
    },
    TypeKindChanged {
        old_type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
        new_type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
    },
    TypeDescriptionChanged {
        old_type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
        new_type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
    },
    FieldAdded {
        added_field_definition: &'a S::FieldDefinition,
        type_name: &'a str,
    },
    FieldRemoved {
        removed_field_definition: &'a S::FieldDefinition,
        type_name: &'a str,
    },
    FieldDescriptionChanged {
        type_name: &'a str,
        old_field_definition: &'a S::FieldDefinition,
        new_field_definition: &'a S::FieldDefinition,
    },
    FieldTypeChanged {
        type_name: &'a str,
        old_field_definition: &'a S::FieldDefinition,
        new_field_definition: &'a S::FieldDefinition,
    },
    FieldArgumentAdded {
        type_name: &'a str,
        field_definition: &'a S::FieldDefinition,
        argument_definition: &'a S::InputValueDefinition,
    },
    FieldArgumentRemoved {
        type_name: &'a str,
        field_definition: &'a S::FieldDefinition,
        argument_definition: &'a S::InputValueDefinition,
    },
    FieldArgumentDescriptionChanged {
        type_name: &'a str,
        field_definition: &'a S::FieldDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    },
    FieldArgumentDefaultValueChanged {
        type_name: &'a str,
        field_definition: &'a S::FieldDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    },
    FieldArgumentTypeChanged {
        type_name: &'a str,
        field_definition: &'a S::FieldDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    },
    ObjectInterfaceAddition {
        object_type_definition: &'a S::ObjectTypeDefinition,
        interface_implementation: &'a S::InterfaceImplementation,
    },
    ObjectInterfaceRemoval {
        object_type_definition: &'a S::ObjectTypeDefinition,
        interface_implementation: &'a S::InterfaceImplementation,
    },
    EnumValueAdded {
        enum_type_definition: &'a S::EnumTypeDefinition,
        enum_value_definition: &'a S::EnumValueDefinition,
    },
    EnumValueRemoved {
        enum_type_definition: &'a S::EnumTypeDefinition,
        enum_value_definition: &'a S::EnumValueDefinition,
    },
    EnumValueDescriptionChanged {
        enum_type_definition: &'a S::EnumTypeDefinition,
        old_enum_value_definition: &'a S::EnumValueDefinition,
        new_enum_value_definition: &'a S::EnumValueDefinition,
    },
    UnionMemberAdded {
        union_type_definition: &'a S::UnionTypeDefinition,
        union_member_type: &'a S::UnionMemberType,
    },
    UnionMemberRemoved {
        union_type_definition: &'a S::UnionTypeDefinition,
        union_member_type: &'a S::UnionMemberType,
    },
    InputFieldAdded {
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        added_field_definition: &'a S::InputValueDefinition,
    },
    InputFieldRemoved {
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        removed_field_definition: &'a S::InputValueDefinition,
    },
    InputFieldDescriptionChanged {
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        old_field_definition: &'a S::InputValueDefinition,
        new_field_definition: &'a S::InputValueDefinition,
    },
    InputFieldTypeChanged {
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        old_field_definition: &'a S::InputValueDefinition,
        new_field_definition: &'a S::InputValueDefinition,
    },
    InputFieldDefaultValueChanged {
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        old_field_definition: &'a S::InputValueDefinition,
        new_field_definition: &'a S::InputValueDefinition,
    },
    DirectiveDefinitionAdded {
        directive_definition: &'a S::DirectiveDefinition,
    },
    DirectiveDefinitionRemoved {
        directive_definition: &'a S::DirectiveDefinition,
    },
    DirectiveDefinitionLocationAdded {
        directive_definition: &'a S::DirectiveDefinition,
        location: &'a DirectiveLocation,
    },
    DirectiveDefinitionLocationRemoved {
        directive_definition: &'a S::DirectiveDefinition,
        location: &'a DirectiveLocation,
    },
    DirectiveDefinitionDescriptionChanged {
        old_directive_definition: &'a S::DirectiveDefinition,
        new_directive_definition: &'a S::DirectiveDefinition,
    },
    DirectiveDefinitionArgumentAdded {
        directive_definition: &'a S::DirectiveDefinition,
        argument_definition: &'a S::InputValueDefinition,
    },
    DirectiveDefinitionArgumentRemoved {
        directive_definition: &'a S::DirectiveDefinition,
        argument_definition: &'a S::InputValueDefinition,
    },
    DirectiveDefinitionArgumentDescriptionChanged {
        directive_definition: &'a S::DirectiveDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    },
    DirectiveDefinitionArgumentDefaultValueChanged {
        directive_definition: &'a S::DirectiveDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    },
    DirectiveDefinitionArgumentTypeChanged {
        directive_definition: &'a S::DirectiveDefinition,
        old_argument_definition: &'a S::InputValueDefinition,
        new_argument_definition: &'a S::InputValueDefinition,
    },
    DirectiveAdded {
        location: DirectiveLocation,
        member_name: &'a str,
        directive: &'a S::Directive,
    },
    DirectiveRemoved {
        location: DirectiveLocation,
        member_name: &'a str,
        directive: &'a S::Directive,
    },
    DirectiveArgumentAdded {
        directive: &'a S::Directive,
        argument: &'a ArgumentForDirective<S>,
    },
    DirectiveArgumentRemoved {
        directive: &'a S::Directive,
        argument: &'a ArgumentForDirective<S>,
    },
    DirectiveArgumentValueChanged {
        directive: &'a S::Directive,
        old_argument: &'a ArgumentForDirective<S>,
        new_argument: &'a ArgumentForDirective<S>,
    },
}

impl<'a, S: SchemaDefinition> Change<'a, S> {
    pub fn breaking(&self) -> bool {
        matches!(self.criticality(), Criticality::Breaking { .. })
    }

    pub fn non_breaking(&self) -> bool {
        matches!(self.criticality(), Criticality::Safe { .. })
    }

    pub fn dangerous(&self) -> bool {
        matches!(self.criticality(), Criticality::Dangerous { .. })
    }

    pub fn criticality(&self) -> Criticality {
        match self {
            Self::TypeRemoved { .. } => Criticality::breaking(
                Some(Cow::from("Removing a type is a breaking change. It is preferable to deprecate and remove all references to this type first."))
            ),
            Self::TypeAdded { .. } => Criticality::safe(None),
            Self::TypeKindChanged { .. } => Criticality::breaking(None),
            Self::TypeDescriptionChanged { .. } => Criticality::safe(None),
            Self::FieldAdded { .. } => Criticality::safe(None),
            Self::FieldRemoved { removed_field_definition: removed_field, type_name: _ } => {
                if removed_field.directives()
                .map_or(
                    false,
                    |directives| directives.iter().any(|d| d.name() == "deprecated"),
                ) {
                    Criticality::breaking(Some(Cow::from("Removing a deprecated field is a breaking change. Before removing it, you may want to look at the field's usage to see the impact of removing the field.")))
                } else {
                    Criticality::breaking(Some(Cow::from("Removing a field is a breaking change. It is preferable to deprecate the field before removing it.")))
                }
            },
            Self::FieldDescriptionChanged { .. } => Criticality::safe(None),
            Self::FieldTypeChanged { type_name: _, old_field_definition: old_field, new_field_definition: new_field } => {
                if is_change_safe_for_field::<S>(old_field.r#type().as_shallow_ref(), new_field.r#type().as_shallow_ref()) {
                    Criticality::safe(None)
                } else {
                    Criticality::breaking(Some(Cow::from("Changing a field's type can cause existing queries that use this field to error.")))
                }
            },
            Self::FieldArgumentAdded { type_name: _, field_definition: _, argument_definition: argument } => {
                if argument.r#type().is_required() && argument.default_value().is_none() {
                    Criticality::breaking(Some(Cow::from("Adding a required argument without a default value to an existing field is a breaking change because it will cause existing uses of this field to error.")))
                } else {
                    Criticality::safe(None)
                }

            },
            Self::FieldArgumentRemoved { .. } => {
                Criticality::breaking(Some(Cow::from("Removing a field argument is a breaking change because it will cause existing queries that use this argument to error.")))
            },
            Self::FieldArgumentDescriptionChanged { .. } => {
                Criticality::safe(None)
            },
            Self::FieldArgumentDefaultValueChanged { .. } => {
                Criticality::dangerous(Some(Cow::from("Changing the default value for an argument may change the runtime behaviour of a field if it was never provided.")))
            },
            Self::FieldArgumentTypeChanged{ type_name: _, field_definition: _, old_argument_definition: old_argument, new_argument_definition: new_argument } => {
                if is_change_safe_for_input_value::<S>(old_argument.r#type().as_shallow_ref(), new_argument.r#type().as_shallow_ref()) {
                    Criticality::safe(None)
                } else {
                    Criticality::breaking(Some(Cow::from("Changing the type of a field's argument can cause existing queries that use this argument to error.")))
                }
            },
            Self::ObjectInterfaceAddition { .. } => {
                Criticality::dangerous(Some(Cow::from("Adding an interface to an object type may break existing clients that were not programming defensively against a new possible type.")))
            },
            Self::ObjectInterfaceRemoval { .. } => {
                Criticality::breaking(Some(Cow::from("Removing an interface from an object type can cause existing queries that use this in a fragment spread to error.")))
            },
            Self::EnumValueAdded { ..} => {
                Criticality::dangerous(Some(Cow::from("Adding an enum value may break existing clients that were not programming defensively against an added case when querying an enum.")))
            },
            Self::EnumValueRemoved { .. } => {
                Criticality::breaking(Some(Cow::from("Removing an enum value will cause existing queries that use this enum value to error.")))
            },
            Self::EnumValueDescriptionChanged { .. } => {
                Criticality::safe(None)
            },
            Self::UnionMemberAdded { .. } => {
                Criticality::dangerous(Some(Cow::from("Adding a possible type to Unions may break existing clients that were not programming defensively against a new possible type..")))
            },
            Self::UnionMemberRemoved { .. } => {
                Criticality::breaking(Some(Cow::from("Removing a union member from a union can cause existing queries that use this union member in a fragment spread to error.")))
            },
            Self::InputFieldAdded { input_object_type_definition: _, added_field_definition: added_field } => {
                if added_field.r#type().is_required() && added_field.default_value().is_none() {
                    Criticality::breaking(Some(Cow::from("Adding a non-null input field without a default value to an existing input type will cause existing queries that use this input type to error because they will not provide a value for this new field.")))
                } else {
                    Criticality::safe(None)
                }
            },
            Self::InputFieldRemoved { .. } => {
                Criticality::breaking(Some(Cow::from("Removing an input field will cause existing queries that use this input field to error.")))
            },
            Self::InputFieldTypeChanged { input_object_type_definition: _, old_field_definition: old_field, new_field_definition: new_field } => {
                if is_change_safe_for_input_value::<S>(old_field.r#type().as_shallow_ref(), new_field.r#type().as_shallow_ref()) {
                    Criticality::safe(Some(Cow::from("Changing an input field from non-null to null is considered non-breaking")))
                } else {
                    Criticality::breaking(Some(Cow::from("Changing the type of an input field can cause existing queries that use this field to error.")))
                }
            },
            Self::InputFieldDescriptionChanged { .. } => {
                Criticality::safe(None)
            },
            Self::InputFieldDefaultValueChanged { .. } => {
                Criticality::dangerous(Some(Cow::from("Changing the default value for an argument may change the runtime behaviour of a field if it was never provided.")))
            },
            Self::DirectiveDefinitionAdded { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveDefinitionRemoved { .. } => {
                Criticality::breaking(None)
            },
            Self::DirectiveDefinitionLocationAdded { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveDefinitionLocationRemoved { .. } => {
                Criticality::breaking(None)
            },
            Self::DirectiveDefinitionDescriptionChanged { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveDefinitionArgumentAdded { directive_definition: _, argument_definition } => {
                if argument_definition.is_required() {
                    Criticality::breaking(None)
                } else {
                    Criticality::safe(None)
                }
            },
            Self::DirectiveDefinitionArgumentRemoved { .. } => {
                Criticality::breaking(None)
            },
            Self::DirectiveDefinitionArgumentDescriptionChanged { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveDefinitionArgumentTypeChanged { directive_definition: _, old_argument_definition, new_argument_definition } => {
                if is_change_safe_for_input_value::<S>(old_argument_definition.r#type().as_shallow_ref(), new_argument_definition.r#type().as_shallow_ref()) {
                    Criticality::safe(Some(Cow::from("Changing an input field from non-null to null is considered non-breaking")))
                } else {
                    Criticality::breaking(None)
                }
            },
            Self::DirectiveDefinitionArgumentDefaultValueChanged { .. } => {
                Criticality::dangerous(Some(Cow::from("Changing the default value for an argument may change the runtime behaviour of a field if it was never provided.")))
            },
            Self::DirectiveAdded { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveRemoved { .. } => {
                Criticality::breaking(None)
            },
            Self::DirectiveArgumentAdded { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveArgumentRemoved { .. } => {
                Criticality::safe(None)
            },
            Self::DirectiveArgumentValueChanged { .. } => {
                Criticality::safe(None)
            },
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::TypeRemoved {
                removed_type_definition: removed_type,
            } => {
                format!("Type `{}` was removed", removed_type.name())
            }
            Self::TypeAdded {
                added_type_definition: added_type,
            } => {
                format!("Type `{}` was added", added_type.name())
            }
            Self::TypeKindChanged {
                old_type_definition: old_type,
                new_type_definition: new_type,
            } => {
                format!(
                    "`{}` kind changed from `{}` to `{}`",
                    old_type.name(),
                    old_type.kind(),
                    new_type.kind()
                )
            }
            Self::TypeDescriptionChanged {
                old_type_definition: old_type,
                new_type_definition: new_type,
            } => {
                format!(
                    "Description `{}` on type `{}` has changed to `{}`",
                    old_type.description().unwrap_or(""),
                    old_type.name(),
                    new_type.description().unwrap_or("")
                )
            }
            Self::FieldAdded {
                added_field_definition: added_field,
                type_name,
            } => {
                format!(
                    "Field `{}` was added to object type `{}`",
                    added_field.name(),
                    type_name
                )
            }
            Self::FieldRemoved {
                removed_field_definition: removed_field,
                type_name,
            } => {
                format!(
                    "Field `{}` was removed from object type `{}`",
                    removed_field.name(),
                    type_name
                )
            }
            Self::FieldDescriptionChanged {
                type_name: _,
                old_field_definition: old_field,
                new_field_definition: new_field,
            } => {
                format!(
                    "Field `{}` description changed from `{}` to `{}`",
                    self.path(),
                    old_field.description().unwrap_or(""),
                    new_field.description().unwrap_or("")
                )
            }
            Self::FieldTypeChanged {
                type_name: _,
                old_field_definition: old_field,
                new_field_definition: new_field,
            } => {
                format!(
                    "Field `{}` changed type from `{}` to `{}`.",
                    self.path(),
                    old_field.r#type().display_name(),
                    new_field.r#type().display_name()
                )
            }
            Self::FieldArgumentAdded {
                type_name,
                field_definition: field,
                argument_definition: argument,
            } => {
                format!(
                    "Argument `{}` was added to field `{}.{}`",
                    argument.name(),
                    type_name,
                    field.name()
                )
            }
            Self::FieldArgumentRemoved {
                type_name,
                field_definition: field,
                argument_definition: argument,
            } => {
                format!(
                    "Argument `{}` was removed from field `{}.{}`",
                    argument.name(),
                    type_name,
                    field.name()
                )
            }
            Self::FieldArgumentDescriptionChanged {
                type_name,
                field_definition: field,
                old_argument_definition: old_argument,
                new_argument_definition: new_argument,
            } => {
                format!(
                    "Description for argument `{}` on field `{}.{}` changed from `{}` to `{}`",
                    new_argument.name(),
                    field.name(),
                    type_name,
                    old_argument.description().unwrap_or(""),
                    new_argument.description().unwrap_or("")
                )
            }
            Self::FieldArgumentDefaultValueChanged {
                type_name,
                field_definition: field,
                old_argument_definition: old_argument,
                new_argument_definition: new_argument,
            } => match (old_argument.default_value(), new_argument.default_value()) {
                (Some(old_default_value), Some(new_default_value)) => {
                    if old_default_value.as_ref() != new_default_value.as_ref() {
                        format!("Default value for argument `{}` on field `{}.{}` was changed from `{}` to `{}`", old_argument.name(), type_name, field.name(), ValuePrinter::new(old_default_value), ValuePrinter::new(new_default_value))
                    } else {
                        String::new()
                    }
                }
                (Some(old_default_value), None) => {
                    format!(
                        "Default value `{}` was removed from argument `{}` on field `{}.{}`",
                        ValuePrinter::new(old_default_value),
                        old_argument.name(),
                        type_name,
                        field.name()
                    )
                }
                (None, Some(new_default_value)) => {
                    format!(
                        "Default value `{}` was added to argument `{}` on field `{}.{}`",
                        ValuePrinter::new(new_default_value),
                        new_argument.name(),
                        type_name,
                        field.name()
                    )
                }
                (None, None) => String::new(),
            },
            Self::FieldArgumentTypeChanged {
                type_name,
                field_definition: field,
                old_argument_definition: old_argument,
                new_argument_definition: new_argument,
            } => {
                format!(
                    "Type for argument `{}` on field `{}.{}` changed from `{}` to `{}`",
                    new_argument.name(),
                    field.name(),
                    type_name,
                    old_argument.r#type().display_name(),
                    new_argument.r#type().display_name()
                )
            }
            Self::ObjectInterfaceAddition {
                object_type_definition: object_type,
                interface_implementation,
            } => {
                format!(
                    "`{}` object implements `{}` interface",
                    object_type.name(),
                    interface_implementation.name()
                )
            }
            Self::ObjectInterfaceRemoval {
                object_type_definition: object_type,
                interface_implementation,
            } => {
                format!(
                    "`{}` object type no longer implements `{}` interface",
                    object_type.name(),
                    interface_implementation.name()
                )
            }
            Self::EnumValueAdded {
                enum_type_definition: enum_type,
                enum_value_definition: enum_value,
            } => {
                format!(
                    "Enum value `{}` was added to enum `{}`",
                    enum_value.name(),
                    enum_type.name()
                )
            }
            Self::EnumValueRemoved {
                enum_type_definition: enum_type,
                enum_value_definition: enum_value,
            } => {
                format!(
                    "Enum value `{}` was removed from enum `{}`",
                    enum_value.name(),
                    enum_type.name()
                )
            }
            Self::EnumValueDescriptionChanged {
                enum_type_definition: enum_type,
                old_enum_value_definition: old_enum_value,
                new_enum_value_definition: new_enum_value,
            } => {
                format!(
                    "Description for enum value `{}.{}` changed from `{}` to `{}`",
                    enum_type.name(),
                    new_enum_value.name(),
                    old_enum_value.description().unwrap_or(""),
                    new_enum_value.description().unwrap_or("")
                )
            }
            Self::UnionMemberAdded {
                union_type_definition,
                union_member_type,
            } => {
                format!(
                    "Union member `{}` was added to union type `{}`",
                    union_member_type.name(),
                    union_type_definition.name()
                )
            }
            Self::UnionMemberRemoved {
                union_type_definition,
                union_member_type,
            } => {
                format!(
                    "Union member `{}` was removed from union type `{}`",
                    union_member_type.name(),
                    union_type_definition.name()
                )
            }
            Self::InputFieldAdded {
                input_object_type_definition: input_object_type,
                added_field_definition: added_field,
            } => {
                format!(
                    "Input field `{}` was added to input object type `{}`",
                    added_field.name(),
                    input_object_type.name()
                )
            }
            Self::InputFieldRemoved {
                input_object_type_definition: input_object_type,
                removed_field_definition: removed_field,
            } => {
                format!(
                    "Input field `{}` was removed from input object type `{}`",
                    removed_field.name(),
                    input_object_type.name()
                )
            }
            Self::InputFieldDescriptionChanged {
                input_object_type_definition: input_object_type,
                old_field_definition: old_field,
                new_field_definition: new_field,
            } => {
                format!(
                    "Input field `{}.{}` description changed from `{}` to `{}`",
                    input_object_type.name(),
                    old_field.name(),
                    old_field.description().unwrap_or(""),
                    new_field.description().unwrap_or("")
                )
            }
            Self::InputFieldTypeChanged {
                input_object_type_definition: input_object_type,
                old_field_definition: old_field,
                new_field_definition: new_field,
            } => {
                format!(
                    "Input field `{}.{}` changed type from `{}` to `{}`",
                    input_object_type.name(),
                    new_field.name(),
                    old_field.r#type().display_name(),
                    new_field.r#type().display_name()
                )
            }
            Self::InputFieldDefaultValueChanged {
                input_object_type_definition,
                old_field_definition,
                new_field_definition,
            } => match (
                old_field_definition.default_value(),
                new_field_definition.default_value(),
            ) {
                (Some(old_default_value), Some(new_default_value)) => {
                    if old_default_value.as_ref() != new_default_value.as_ref() {
                        format!(
                            "Input field `{}.{}` default value changed from `{}` to `{}`",
                            input_object_type_definition.name(),
                            new_field_definition.name(),
                            ValuePrinter::new(old_default_value),
                            ValuePrinter::new(new_default_value)
                        )
                    } else {
                        String::new()
                    }
                }
                (Some(old_default_value), None) => {
                    format!(
                        "Default value `{}` was removed from input field `{}.{}`",
                        ValuePrinter::new(old_default_value),
                        input_object_type_definition.name(),
                        old_field_definition.name()
                    )
                }
                (None, Some(new_default_value)) => {
                    format!(
                        "Default value `{}` was added to input field `{}.{}`",
                        ValuePrinter::new(new_default_value),
                        input_object_type_definition.name(),
                        old_field_definition.name()
                    )
                }
                (None, None) => String::new(),
            },
            Self::DirectiveDefinitionAdded {
                directive_definition: directive,
            } => {
                format!("Directive `{}` was added", directive.name())
            }
            Self::DirectiveDefinitionRemoved {
                directive_definition,
            } => {
                format!("Directive `{}` was removed", directive_definition.name())
            }
            Self::DirectiveDefinitionLocationAdded {
                directive_definition,
                location,
            } => {
                format!(
                    "Location `{}` was added to directive `{}`",
                    location,
                    directive_definition.name()
                )
            }
            Self::DirectiveDefinitionLocationRemoved {
                directive_definition,
                location,
            } => {
                format!(
                    "Location `{}` was removed from directive `{}`",
                    location,
                    directive_definition.name()
                )
            }
            Self::DirectiveDefinitionDescriptionChanged {
                old_directive_definition: old_directive,
                new_directive_definition: new_directive,
            } => {
                format!(
                    "Directive `{}` description changed from `{}` to `{}`",
                    new_directive.name(),
                    old_directive.description().unwrap_or(""),
                    new_directive.description().unwrap_or("")
                )
            }
            Self::DirectiveDefinitionArgumentAdded {
                directive_definition,
                argument_definition,
            } => {
                format!(
                    "Argument `{}` was added to directive `{}`",
                    argument_definition.name(),
                    directive_definition.name()
                )
            }
            Self::DirectiveDefinitionArgumentRemoved {
                directive_definition,
                argument_definition,
            } => {
                format!(
                    "Argument `{}` was removed from directive `{}`",
                    argument_definition.name(),
                    directive_definition.name()
                )
            }
            Self::DirectiveDefinitionArgumentDescriptionChanged {
                directive_definition,
                old_argument_definition,
                new_argument_definition,
            } => {
                format!(
                    "Description for argument `{}` on directive `{}` changed from `{}` to `{}`",
                    new_argument_definition.name(),
                    directive_definition.name(),
                    old_argument_definition.description().unwrap_or(""),
                    new_argument_definition.description().unwrap_or("")
                )
            }
            Self::DirectiveDefinitionArgumentTypeChanged {
                directive_definition,
                old_argument_definition,
                new_argument_definition,
            } => {
                format!(
                    "Type for argument `{}` on directive `{}` changed from `{}` to `{}`",
                    new_argument_definition.name(),
                    directive_definition.name(),
                    old_argument_definition.r#type().display_name(),
                    new_argument_definition.r#type().display_name()
                )
            }
            Self::DirectiveDefinitionArgumentDefaultValueChanged {
                directive_definition,
                old_argument_definition,
                new_argument_definition,
            } => {
                match (
                    old_argument_definition.default_value(),
                    new_argument_definition.default_value(),
                ) {
                    (Some(old_default_value), Some(new_default_value)) => {
                        if old_default_value.as_ref() != new_default_value.as_ref() {
                            format!("Directive argument `{}.{}` default value changed from `{}` to `{}`", directive_definition.name(), new_argument_definition.name(), ValuePrinter::new(old_default_value), ValuePrinter::new(new_default_value))
                        } else {
                            String::new()
                        }
                    }
                    (Some(old_default_value), None) => {
                        format!(
                            "Default value `{}` was removed from directive argument `{}.{}`",
                            ValuePrinter::new(old_default_value),
                            directive_definition.name(),
                            old_argument_definition.name()
                        )
                    }
                    (None, Some(new_default_value)) => {
                        format!(
                            "Default value `{}` was added to directive argument `{}.{}`",
                            ValuePrinter::new(new_default_value),
                            directive_definition.name(),
                            old_argument_definition.name()
                        )
                    }
                    (None, None) => String::new(),
                }
            }
            Self::DirectiveAdded {
                location,
                member_name,
                directive,
            } => {
                format!(
                    "Directive `{}` was added to {} `{}`",
                    directive.name(),
                    directive_location_name(location),
                    member_name
                )
            }
            Self::DirectiveRemoved {
                location,
                member_name,
                directive,
            } => {
                format!(
                    "Directive `{}` was removed from {} `{}`",
                    directive.name(),
                    directive_location_name(location),
                    member_name
                )
            }
            Self::DirectiveArgumentAdded {
                directive,
                argument,
            } => {
                format!(
                    "Argument `{}` was added to directive `{}`",
                    argument.name(),
                    directive.name()
                )
            }
            Self::DirectiveArgumentRemoved {
                directive,
                argument,
            } => {
                format!(
                    "Argument `{}` was removed from directive `{}`",
                    argument.name(),
                    directive.name()
                )
            }
            Self::DirectiveArgumentValueChanged {
                directive,
                old_argument,
                new_argument,
            } => {
                format!(
                    "Value for argument `{}` on directive `{}` changed from {} to {}",
                    new_argument.name(),
                    directive.name(),
                    ValuePrinter::new(old_argument.value()),
                    ValuePrinter::new(new_argument.value()),
                )
            }
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::TypeRemoved {
                removed_type_definition: removed_type,
            } => removed_type.name().to_string(),
            Self::TypeAdded {
                added_type_definition: added_type,
            } => added_type.name().to_string(),
            Self::TypeKindChanged {
                old_type_definition: old_type,
                new_type_definition: _,
            } => old_type.name().to_string(),
            Self::TypeDescriptionChanged {
                old_type_definition: old_type,
                new_type_definition: _,
            } => old_type.name().to_string(),
            Self::FieldAdded {
                added_field_definition: added_field,
                type_name,
            } => [type_name, added_field.name()].join("."),
            Self::FieldRemoved {
                removed_field_definition: removed_field,
                type_name,
            } => [type_name, removed_field.name()].join("."),
            Self::FieldDescriptionChanged {
                type_name,
                old_field_definition: old_field,
                new_field_definition: _,
            } => [type_name, old_field.name()].join("."),
            Self::FieldTypeChanged {
                type_name,
                old_field_definition: old_field,
                new_field_definition: _,
            } => [type_name, old_field.name()].join("."),
            Self::FieldArgumentAdded {
                type_name,
                field_definition: field,
                argument_definition: argument,
            } => [type_name, field.name(), argument.name()].join("."),
            Self::FieldArgumentRemoved {
                type_name,
                field_definition: field,
                argument_definition: argument,
            } => [type_name, field.name(), argument.name()].join("."),
            Self::FieldArgumentDescriptionChanged {
                type_name,
                field_definition: field,
                old_argument_definition: old_argument,
                new_argument_definition: _,
            } => [type_name, field.name(), old_argument.name()].join("."),
            Self::FieldArgumentDefaultValueChanged {
                type_name,
                field_definition: field,
                old_argument_definition: old_argument,
                new_argument_definition: _,
            } => [type_name, field.name(), old_argument.name()].join("."),
            Self::FieldArgumentTypeChanged {
                type_name,
                field_definition: field,
                old_argument_definition: old_argument,
                new_argument_definition: _,
            } => [type_name, field.name(), old_argument.name()].join("."),
            Self::ObjectInterfaceAddition {
                object_type_definition,
                ..
            } => object_type_definition.name().to_string(),
            Self::ObjectInterfaceRemoval {
                object_type_definition,
                ..
            } => object_type_definition.name().to_string(),
            Self::EnumValueAdded {
                enum_type_definition: enum_type,
                enum_value_definition: enum_value,
            } => [enum_type.name(), enum_value.name()].join("."),
            Self::EnumValueRemoved {
                enum_type_definition: enum_type,
                enum_value_definition: enum_value,
            } => [enum_type.name(), enum_value.name()].join("."),
            Self::EnumValueDescriptionChanged {
                enum_type_definition: enum_type,
                old_enum_value_definition: old_enum_value,
                new_enum_value_definition: _,
            } => [enum_type.name(), old_enum_value.name()].join("."),
            Self::UnionMemberAdded {
                union_type_definition,
                ..
            } => union_type_definition.name().to_string(),
            Self::UnionMemberRemoved {
                union_type_definition,
                ..
            } => union_type_definition.name().to_string(),
            Self::InputFieldAdded {
                input_object_type_definition: input_object_type,
                added_field_definition: added_field,
            } => [input_object_type.name(), added_field.name()].join("."),
            Self::InputFieldRemoved {
                input_object_type_definition: input_object_type,
                removed_field_definition: removed_field,
            } => [input_object_type.name(), removed_field.name()].join("."),
            Self::InputFieldDescriptionChanged {
                input_object_type_definition: input_object_type,
                old_field_definition: old_field,
                new_field_definition: _,
            } => [input_object_type.name(), old_field.name()].join("."),
            Self::InputFieldTypeChanged {
                input_object_type_definition: input_object_type,
                old_field_definition: old_field,
                new_field_definition: _,
            } => [input_object_type.name(), old_field.name()].join("."),
            Self::InputFieldDefaultValueChanged {
                input_object_type_definition,
                old_field_definition,
                new_field_definition: _,
            } => [
                input_object_type_definition.name(),
                old_field_definition.name(),
            ]
            .join("."),
            Self::DirectiveDefinitionAdded {
                directive_definition: directive,
            } => format!("@{}", directive.name()),
            Self::DirectiveDefinitionRemoved {
                directive_definition,
            } => {
                format!("@{}", directive_definition.name())
            }
            Self::DirectiveDefinitionLocationAdded {
                directive_definition,
                location: _,
            } => {
                format!("@{}", directive_definition.name())
            }
            Self::DirectiveDefinitionLocationRemoved {
                directive_definition,
                location: _,
            } => {
                format!("@{}", directive_definition.name())
            }
            Self::DirectiveDefinitionDescriptionChanged {
                old_directive_definition: _,
                new_directive_definition,
            } => {
                format!("@{}", new_directive_definition.name())
            }
            Self::DirectiveDefinitionArgumentAdded {
                directive_definition,
                argument_definition,
            } => {
                format!(
                    "@{}.{}",
                    directive_definition.name(),
                    argument_definition.name()
                )
            }
            Self::DirectiveDefinitionArgumentRemoved {
                directive_definition,
                argument_definition,
            } => {
                format!(
                    "@{}.{}",
                    directive_definition.name(),
                    argument_definition.name()
                )
            }
            Self::DirectiveDefinitionArgumentDescriptionChanged {
                directive_definition,
                old_argument_definition,
                new_argument_definition: _,
            } => {
                format!(
                    "@{}.{}",
                    directive_definition.name(),
                    old_argument_definition.name()
                )
            }
            Self::DirectiveDefinitionArgumentTypeChanged {
                directive_definition,
                old_argument_definition,
                new_argument_definition: _,
            } => {
                format!(
                    "@{}.{}",
                    directive_definition.name(),
                    old_argument_definition.name()
                )
            }
            Self::DirectiveDefinitionArgumentDefaultValueChanged {
                directive_definition,
                old_argument_definition,
                new_argument_definition: _,
            } => {
                format!(
                    "@{}.{}",
                    directive_definition.name(),
                    old_argument_definition.name()
                )
            }
            Self::DirectiveAdded {
                location: _,
                member_name: _,
                directive,
            }
            | Self::DirectiveRemoved {
                location: _,
                member_name: _,
                directive,
            } => directive.name().to_string(),
            Self::DirectiveArgumentAdded {
                directive,
                argument,
            } => {
                format!("@{}.{}", directive.name(), argument.name())
            }
            Self::DirectiveArgumentRemoved {
                directive,
                argument,
            } => {
                format!("@{}.{}", directive.name(), argument.name())
            }
            Self::DirectiveArgumentValueChanged {
                directive,
                old_argument,
                new_argument: _,
            } => {
                format!("@{}.{}", directive.name(), old_argument.name())
            }
        }
    }
}

fn is_change_safe_for_field<S: SchemaDefinition>(
    old_type: ShallowOutputTypeReference<S::OutputType>,
    new_type: ShallowOutputTypeReference<S::OutputType>,
) -> bool {
    match (old_type, new_type) {
        (
            ShallowOutputTypeReference::Base(old_base, old_required),
            ShallowOutputTypeReference::Base(new_base, new_required),
        ) => (!old_required || new_required) && old_base == new_base,
        (
            ShallowOutputTypeReference::List(old_inner, old_required),
            ShallowOutputTypeReference::List(new_inner, new_required),
        ) => {
            (!old_required || new_required)
                && is_change_safe_for_field::<S>(
                    old_inner.as_shallow_ref(),
                    new_inner.as_shallow_ref(),
                )
        }
        _ => false,
    }
}

fn is_change_safe_for_input_value<S: SchemaDefinition>(
    old_type: ShallowInputTypeReference<S::InputType>,
    new_type: ShallowInputTypeReference<S::InputType>,
) -> bool {
    match (old_type, new_type) {
        (
            ShallowInputTypeReference::Base(old_base, old_required),
            ShallowInputTypeReference::Base(new_base, new_required),
        ) => (old_required || !new_required) && old_base == new_base,
        (
            ShallowInputTypeReference::List(old_inner, old_required),
            ShallowInputTypeReference::List(new_inner, new_required),
        ) => {
            (old_required || !new_required)
                && is_change_safe_for_input_value::<S>(
                    old_inner.as_shallow_ref(),
                    new_inner.as_shallow_ref(),
                )
        }
        _ => false,
    }
}

fn directive_location_name(location: &DirectiveLocation) -> &'static str {
    match location {
        DirectiveLocation::ArgumentDefinition => "argument",
        DirectiveLocation::EnumValue => "enum value",
        DirectiveLocation::FieldDefinition => "field",
        DirectiveLocation::InputFieldDefinition => "input field",
        DirectiveLocation::InputObject => "input object",
        DirectiveLocation::Interface => "interface",
        DirectiveLocation::Object => "object",
        DirectiveLocation::Scalar => "scalar",
        DirectiveLocation::Schema => "schema",
        DirectiveLocation::Union => "union",
        _ => "unknown",
    }
}
