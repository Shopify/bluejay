use crate::format::Formatter;
use bluejay_core::definition::HasDirectives;
use bluejay_core::definition::{
    ArgumentsDefinition, DirectiveDefinition, EnumTypeDefinition, EnumValueDefinition,
    FieldDefinition, FieldsDefinition, InputObjectTypeDefinition, InputType, InputValueDefinition,
    InterfaceImplementation, InterfaceImplementations, InterfaceTypeDefinition,
    ObjectTypeDefinition, OutputType, ScalarTypeDefinition, SchemaDefinition,
    TypeDefinitionReference, UnionMemberType, UnionTypeDefinition,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinition, Selection, SelectionReference, SelectionSet, VariableDefinition,
    VariableDefinitions, VariableType,
};
use bluejay_core::{
    Argument, Arguments, AsIter, Directive, Directives, ObjectValue, Value, ValueReference,
    Variable,
};
use std::fmt;

pub struct Serializer<W: fmt::Write, F: Formatter> {
    writer: W,
    formatter: F,
}

impl<F: Formatter> Serializer<String, F> {
    pub fn to_string(formatter: F) -> Self {
        Self::new(String::new(), formatter)
    }
}

impl<W: fmt::Write, F: Formatter> Serializer<W, F> {
    pub fn new(writer: W, formatter: F) -> Self {
        Self { writer, formatter }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    // ── Value serialization ──────────────────────────────────────────

    pub fn serialize_value<const CONST: bool, V: Value<CONST>>(
        &mut self,
        value: &V,
    ) -> fmt::Result {
        match value.as_ref() {
            ValueReference::Boolean(b) => write!(self.writer, "{b}"),
            ValueReference::Enum(e) => write!(self.writer, "{e}"),
            ValueReference::Float(fl) => {
                if fl.fract().abs() < 1e-10 {
                    write!(self.writer, "{fl:.1}")
                } else {
                    write!(self.writer, "{fl}")
                }
            }
            ValueReference::Integer(i) => write!(self.writer, "{i}"),
            ValueReference::List(l) => {
                write!(self.writer, "[")?;
                for (idx, el) in l.iter().enumerate() {
                    if idx != 0 {
                        self.formatter.write_list_separator(&mut self.writer)?;
                    }
                    self.serialize_value(el)?;
                }
                write!(self.writer, "]")
            }
            ValueReference::Null => write!(self.writer, "null"),
            ValueReference::Object(o) => {
                self.formatter.begin_value_object(&mut self.writer)?;
                for (idx, (key, value)) in o.iter().enumerate() {
                    if idx != 0 {
                        self.formatter.write_list_separator(&mut self.writer)?;
                    }
                    write!(self.writer, "{}", key.as_ref())?;
                    self.formatter.write_key_value_separator(&mut self.writer)?;
                    self.serialize_value(value)?;
                }
                self.formatter.end_value_object(&mut self.writer)
            }
            ValueReference::String(s) => self.serialize_string_value(s),
            ValueReference::Variable(v) => write!(self.writer, "${}", v.name()),
        }
    }

    fn serialize_string_value(&mut self, value: &str) -> fmt::Result {
        write!(self.writer, "\"")?;
        for c in value.chars() {
            match c {
                '\"' | '\\' => write!(self.writer, "\\{c}")?,
                '\u{0008}' => write!(self.writer, "\\b")?,
                '\u{000C}' => write!(self.writer, "\\f")?,
                '\n' => write!(self.writer, "\\n")?,
                '\r' => write!(self.writer, "\\r")?,
                '\t' => write!(self.writer, "\\t")?,
                c => write!(self.writer, "{c}")?,
            }
        }
        write!(self.writer, "\"")
    }

    // ── Argument serialization ───────────────────────────────────────

    fn serialize_argument<const CONST: bool, A: Argument<CONST>>(
        &mut self,
        argument: &A,
    ) -> fmt::Result {
        write!(self.writer, "{}", argument.name())?;
        self.formatter.write_key_value_separator(&mut self.writer)?;
        self.serialize_value(argument.value())
    }

    fn serialize_arguments<const CONST: bool, A: Arguments<CONST>>(
        &mut self,
        arguments: &A,
    ) -> fmt::Result {
        if arguments.is_empty() {
            return Ok(());
        }
        write!(self.writer, "(")?;
        for (idx, argument) in arguments.iter().enumerate() {
            if idx != 0 {
                self.formatter.write_list_separator(&mut self.writer)?;
            }
            self.serialize_argument(argument)?;
        }
        write!(self.writer, ")")
    }

    // ── Directive serialization ──────────────────────────────────────

    fn serialize_directive<const CONST: bool, D: Directive<CONST>>(
        &mut self,
        directive: &D,
    ) -> fmt::Result {
        write!(self.writer, "@{}", directive.name())?;
        if let Some(arguments) = directive.arguments() {
            self.serialize_arguments(arguments)?;
        }
        Ok(())
    }

    fn serialize_directives<const CONST: bool, D: Directives<CONST>>(
        &mut self,
        directives: &D,
    ) -> fmt::Result {
        for directive in directives.iter() {
            write!(self.writer, " ")?;
            self.serialize_directive(directive)?;
        }
        Ok(())
    }

    // ── Schema definition serialization ──────────────────────────────

    pub fn serialize_schema_definition<S: SchemaDefinition>(
        &mut self,
        schema_definition: &S,
    ) -> fmt::Result {
        let mut had_directives = false;
        for (idx, dd) in schema_definition
            .directive_definitions()
            .filter(|dd| !dd.is_builtin())
            .enumerate()
        {
            if idx != 0 {
                self.formatter
                    .write_definition_separator(&mut self.writer)?;
            }
            self.serialize_directive_definition(dd)?;
            had_directives = true;
        }

        let mut had_types = false;
        for (idx, tdr) in schema_definition
            .type_definitions()
            .filter(|tdr| !tdr.is_builtin())
            .enumerate()
        {
            if had_directives || idx != 0 {
                self.formatter
                    .write_definition_separator(&mut self.writer)?;
            }
            match tdr {
                TypeDefinitionReference::BuiltinScalar(_) => {}
                TypeDefinitionReference::CustomScalar(cstd) => {
                    self.serialize_scalar_type_definition(cstd)?;
                }
                TypeDefinitionReference::Enum(etd) => {
                    self.serialize_enum_type_definition(etd)?;
                }
                TypeDefinitionReference::InputObject(iotd) => {
                    self.serialize_input_object_type_definition(iotd)?;
                }
                TypeDefinitionReference::Interface(itd) => {
                    self.serialize_interface_type_definition(itd)?;
                }
                TypeDefinitionReference::Object(otd) => {
                    self.serialize_object_type_definition(otd)?;
                }
                TypeDefinitionReference::Union(utd) => {
                    self.serialize_union_type_definition(utd)?;
                }
            }
            had_types = true;
        }

        if !Self::is_schema_implicit(schema_definition) {
            if had_directives || had_types {
                self.formatter
                    .write_definition_separator(&mut self.writer)?;
            }
            self.serialize_explicit_schema_definition(schema_definition)?;
        }

        Ok(())
    }

    fn is_schema_implicit<S: SchemaDefinition>(schema_definition: &S) -> bool {
        schema_definition.description().is_none()
            && schema_definition.query().name() == "Query"
            && schema_definition
                .mutation()
                .map(|mutation| mutation.name() == "Mutation")
                .unwrap_or(true)
            && schema_definition
                .subscription()
                .map(|subscription| subscription.name() == "Subscription")
                .unwrap_or(true)
            && schema_definition
                .directives()
                .map(AsIter::is_empty)
                .unwrap_or(true)
    }

    fn serialize_explicit_schema_definition<S: SchemaDefinition>(
        &mut self,
        schema_definition: &S,
    ) -> fmt::Result {
        if let Some(description) = schema_definition.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "schema")?;

        if let Some(directives) = schema_definition.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_space_before_block(&mut self.writer)?;
        self.formatter.begin_block(&mut self.writer, 0)?;
        self.formatter.write_indent(&mut self.writer, 1)?;
        write!(self.writer, "query: {}", schema_definition.query().name())?;
        self.formatter.write_line_ending(&mut self.writer)?;

        if let Some(mutation) = schema_definition.mutation() {
            self.formatter
                .write_block_item_separator(&mut self.writer)?;
            self.formatter.write_indent(&mut self.writer, 1)?;
            write!(self.writer, "mutation: {}", mutation.name())?;
            self.formatter.write_line_ending(&mut self.writer)?;
        }

        if let Some(subscription) = schema_definition.subscription() {
            self.formatter
                .write_block_item_separator(&mut self.writer)?;
            self.formatter.write_indent(&mut self.writer, 1)?;
            write!(self.writer, "subscription: {}", subscription.name())?;
            self.formatter.write_line_ending(&mut self.writer)?;
        }

        self.formatter.end_block(&mut self.writer, 0)?;
        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Scalar type definition ───────────────────────────────────────

    fn serialize_scalar_type_definition<S: ScalarTypeDefinition>(
        &mut self,
        scalar: &S,
    ) -> fmt::Result {
        if let Some(description) = scalar.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "scalar {}", scalar.name())?;

        if let Some(directives) = scalar.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Object type definition ───────────────────────────────────────

    fn serialize_object_type_definition<O: ObjectTypeDefinition>(
        &mut self,
        otd: &O,
    ) -> fmt::Result {
        if let Some(description) = otd.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "type {}", otd.name())?;

        if let Some(interface_implementations) = otd.interface_implementations() {
            self.serialize_interface_implementations(interface_implementations)?;
        }

        if let Some(directives) = otd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_space_before_block(&mut self.writer)?;
        self.serialize_fields_definition(otd.fields_definition())
    }

    // ── Interface type definition ────────────────────────────────────

    fn serialize_interface_type_definition<I: InterfaceTypeDefinition>(
        &mut self,
        itd: &I,
    ) -> fmt::Result {
        if let Some(description) = itd.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "interface {}", itd.name())?;

        if let Some(interface_implementations) = itd.interface_implementations() {
            self.serialize_interface_implementations(interface_implementations)?;
        }

        if let Some(directives) = itd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_space_before_block(&mut self.writer)?;
        self.serialize_fields_definition(itd.fields_definition())
    }

    // ── Union type definition ────────────────────────────────────────

    fn serialize_union_type_definition<U: UnionTypeDefinition>(&mut self, utd: &U) -> fmt::Result {
        if let Some(description) = utd.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "union {}", utd.name())?;

        if let Some(directives) = utd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_equals(&mut self.writer)?;

        for (idx, union_member) in utd.union_member_types().iter().enumerate() {
            if idx != 0 {
                self.formatter.write_union_separator(&mut self.writer)?;
            }
            write!(self.writer, "{}", union_member.name())?;
        }

        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Enum type definition ─────────────────────────────────────────

    fn serialize_enum_type_definition<E: EnumTypeDefinition>(&mut self, etd: &E) -> fmt::Result {
        if let Some(description) = etd.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "enum {}", etd.name())?;

        if let Some(directives) = etd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_space_before_block(&mut self.writer)?;
        self.formatter.begin_block(&mut self.writer, 0)?;

        for (idx, evd) in etd.enum_value_definitions().iter().enumerate() {
            if idx != 0 {
                self.formatter
                    .write_block_item_separator(&mut self.writer)?;
            }

            if let Some(description) = evd.description() {
                self.formatter
                    .write_description(&mut self.writer, description, 1)?;
            }

            self.formatter.write_indent(&mut self.writer, 1)?;
            write!(self.writer, "{}", evd.name())?;

            if let Some(directives) = evd.directives() {
                self.serialize_directives(directives)?;
            }

            self.formatter.write_line_ending(&mut self.writer)?;
        }

        self.formatter.end_block(&mut self.writer, 0)?;
        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Input object type definition ─────────────────────────────────

    fn serialize_input_object_type_definition<I: InputObjectTypeDefinition>(
        &mut self,
        iotd: &I,
    ) -> fmt::Result {
        if let Some(description) = iotd.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "input {}", iotd.name())?;

        if let Some(directives) = iotd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_space_before_block(&mut self.writer)?;
        self.formatter.begin_block(&mut self.writer, 0)?;

        for (idx, ivd) in iotd.input_field_definitions().iter().enumerate() {
            if idx != 0 {
                self.formatter
                    .write_block_item_separator(&mut self.writer)?;
            }
            self.serialize_input_value_definition(ivd, 1)?;
        }

        self.formatter.end_block(&mut self.writer, 0)?;
        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Field definition ─────────────────────────────────────────────

    fn serialize_field_definition<FD: FieldDefinition>(
        &mut self,
        fd: &FD,
        depth: usize,
    ) -> fmt::Result {
        if let Some(description) = fd.description() {
            self.formatter
                .write_description(&mut self.writer, description, depth)?;
        }

        self.formatter.write_indent(&mut self.writer, depth)?;
        write!(self.writer, "{}", fd.name())?;

        if let Some(arguments_definition) = fd.arguments_definition() {
            self.serialize_arguments_definition(arguments_definition, depth)?;
        }

        self.formatter.write_key_value_separator(&mut self.writer)?;
        write!(self.writer, "{}", fd.r#type().display_name())?;

        if let Some(directives) = fd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_line_ending(&mut self.writer)
    }

    fn serialize_fields_definition<FD: FieldsDefinition>(
        &mut self,
        fields_definition: &FD,
    ) -> fmt::Result {
        self.formatter.begin_block(&mut self.writer, 0)?;

        for (idx, fd) in fields_definition
            .iter()
            .filter(|fd| !fd.is_builtin())
            .enumerate()
        {
            if idx != 0 {
                self.formatter
                    .write_block_item_separator(&mut self.writer)?;
            }
            self.serialize_field_definition(fd, 1)?;
        }

        self.formatter.end_block(&mut self.writer, 0)?;
        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Input value definition ───────────────────────────────────────

    fn serialize_input_value_definition<I: InputValueDefinition>(
        &mut self,
        ivd: &I,
        depth: usize,
    ) -> fmt::Result {
        if let Some(description) = ivd.description() {
            self.formatter
                .write_description(&mut self.writer, description, depth)?;
        }

        self.formatter.write_indent(&mut self.writer, depth)?;
        write!(self.writer, "{}", ivd.name())?;
        self.formatter.write_key_value_separator(&mut self.writer)?;
        write!(self.writer, "{}", ivd.r#type().display_name())?;

        if let Some(default_value) = ivd.default_value() {
            self.formatter.write_equals(&mut self.writer)?;
            self.serialize_value(default_value)?;
        }

        if let Some(directives) = ivd.directives() {
            self.serialize_directives(directives)?;
        }

        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Arguments definition ─────────────────────────────────────────

    fn serialize_arguments_definition<AD: ArgumentsDefinition>(
        &mut self,
        arguments_definition: &AD,
        depth: usize,
    ) -> fmt::Result {
        if arguments_definition.is_empty() {
            return Ok(());
        }

        write!(self.writer, "(")?;
        self.formatter.write_line_ending(&mut self.writer)?;

        for (idx, ivd) in arguments_definition.iter().enumerate() {
            if idx != 0 {
                self.formatter
                    .write_block_item_separator(&mut self.writer)?;
            }
            self.serialize_input_value_definition(ivd, depth + 1)?;
        }

        self.formatter.write_indent(&mut self.writer, depth)?;
        write!(self.writer, ")")
    }

    // ── Directive definition ─────────────────────────────────────────

    fn serialize_directive_definition<DD: DirectiveDefinition>(&mut self, dd: &DD) -> fmt::Result {
        if let Some(description) = dd.description() {
            self.formatter
                .write_description(&mut self.writer, description, 0)?;
        }

        write!(self.writer, "directive @{}", dd.name())?;

        if let Some(arguments_definition) = dd.arguments_definition() {
            self.serialize_arguments_definition(arguments_definition, 0)?;
        }

        if dd.is_repeatable() {
            write!(self.writer, " repeatable")?;
        }

        write!(self.writer, " on ")?;

        for (idx, location) in dd.locations().iter().enumerate() {
            if idx != 0 {
                self.formatter.write_union_separator(&mut self.writer)?;
            }
            write!(self.writer, "{location}")?;
        }

        self.formatter.write_line_ending(&mut self.writer)
    }

    // ── Interface implementations ────────────────────────────────────

    fn serialize_interface_implementations<II: InterfaceImplementations>(
        &mut self,
        interface_implementations: &II,
    ) -> fmt::Result {
        if !interface_implementations.is_empty() {
            write!(self.writer, " implements ")?;
            for (idx, ii) in interface_implementations.iter().enumerate() {
                if idx != 0 {
                    write!(self.writer, " & ")?;
                }
                write!(self.writer, "{}", ii.name())?;
            }
        }
        Ok(())
    }

    // ── Executable document serialization ────────────────────────────

    pub fn serialize_executable_document<T: ExecutableDocument>(&mut self, doc: &T) -> fmt::Result {
        for (idx, operation_definition) in doc.operation_definitions().enumerate() {
            if idx != 0 {
                self.formatter
                    .write_definition_separator(&mut self.writer)?;
            }
            self.serialize_operation_definition(operation_definition)?;
            self.formatter.write_line_ending(&mut self.writer)?;
        }

        for fragment_definition in doc.fragment_definitions() {
            self.formatter
                .write_definition_separator(&mut self.writer)?;
            self.serialize_fragment_definition(fragment_definition)?;
            self.formatter.write_line_ending(&mut self.writer)?;
        }

        Ok(())
    }

    // ── Operation definition ─────────────────────────────────────────

    fn serialize_operation_definition<O: OperationDefinition>(
        &mut self,
        operation_definition: &O,
    ) -> fmt::Result {
        let odr = operation_definition.as_ref();
        write!(self.writer, "{}", odr.operation_type())?;
        if let Some(name) = odr.name() {
            write!(self.writer, " {name}")?;
        }
        if let Some(variable_definitions) = odr.variable_definitions() {
            self.serialize_variable_definitions(variable_definitions)?;
        }
        if let Some(directives) = odr.directives() {
            self.serialize_directives(directives)?;
        }
        self.formatter.write_space_before_block(&mut self.writer)?;
        self.serialize_selection_set(odr.selection_set(), 0)
    }

    // ── Selection set ────────────────────────────────────────────────

    fn serialize_selection_set<SS: SelectionSet>(
        &mut self,
        selection_set: &SS,
        depth: usize,
    ) -> fmt::Result {
        self.formatter.begin_block(&mut self.writer, depth)?;
        for (idx, selection) in selection_set.iter().enumerate() {
            if idx != 0 {
                self.formatter
                    .write_block_item_separator(&mut self.writer)?;
            }
            self.serialize_selection(selection, depth + 1)?;
            self.formatter.write_line_ending(&mut self.writer)?;
        }
        self.formatter.end_block(&mut self.writer, depth)
    }

    // ── Selection ────────────────────────────────────────────────────

    fn serialize_selection<S: Selection>(&mut self, selection: &S, depth: usize) -> fmt::Result {
        match selection.as_ref() {
            SelectionReference::Field(field) => self.serialize_field(field, depth),
            SelectionReference::FragmentSpread(fragment_spread) => {
                self.serialize_fragment_spread(fragment_spread, depth)
            }
            SelectionReference::InlineFragment(inline_fragment) => {
                self.serialize_inline_fragment(inline_fragment, depth)
            }
        }
    }

    // ── Field (executable) ───────────────────────────────────────────

    fn serialize_field<FE: Field>(&mut self, field: &FE, depth: usize) -> fmt::Result {
        self.formatter.write_indent(&mut self.writer, depth)?;
        if let Some(alias) = field.alias() {
            write!(self.writer, "{alias}")?;
            self.formatter.write_key_value_separator(&mut self.writer)?;
        }
        write!(self.writer, "{}", field.name())?;
        if let Some(arguments) = field.arguments() {
            self.serialize_arguments(arguments)?;
        }
        if let Some(directives) = field.directives() {
            self.serialize_directives(directives)?;
        }
        if let Some(selection_set) = field.selection_set() {
            self.formatter.write_space_before_block(&mut self.writer)?;
            self.serialize_selection_set(selection_set, depth)?;
        }
        Ok(())
    }

    // ── Fragment definition ──────────────────────────────────────────

    fn serialize_fragment_definition<FD: FragmentDefinition>(&mut self, fd: &FD) -> fmt::Result {
        write!(
            self.writer,
            "fragment {} on {}",
            fd.name(),
            fd.type_condition(),
        )?;
        self.formatter.write_space_before_block(&mut self.writer)?;
        self.serialize_selection_set(fd.selection_set(), 0)
    }

    // ── Fragment spread ──────────────────────────────────────────────

    fn serialize_fragment_spread<FS: FragmentSpread>(
        &mut self,
        fragment_spread: &FS,
        depth: usize,
    ) -> fmt::Result {
        self.formatter.write_indent(&mut self.writer, depth)?;
        write!(self.writer, "...{}", fragment_spread.name())?;
        if let Some(directives) = fragment_spread.directives() {
            self.serialize_directives(directives)?;
        }
        Ok(())
    }

    // ── Inline fragment ──────────────────────────────────────────────

    fn serialize_inline_fragment<IF: InlineFragment>(
        &mut self,
        inline_fragment: &IF,
        depth: usize,
    ) -> fmt::Result {
        self.formatter.write_indent(&mut self.writer, depth)?;
        write!(self.writer, "...")?;
        if let Some(type_condition) = inline_fragment.type_condition() {
            write!(self.writer, " on {type_condition}")?;
        }
        if let Some(directives) = inline_fragment.directives() {
            self.serialize_directives(directives)?;
        }
        self.formatter.write_space_before_block(&mut self.writer)?;
        self.serialize_selection_set(inline_fragment.selection_set(), depth)
    }

    // ── Variable definitions ─────────────────────────────────────────

    fn serialize_variable_definition<VD: VariableDefinition>(&mut self, vd: &VD) -> fmt::Result {
        write!(self.writer, "${}", vd.variable())?;
        self.formatter.write_key_value_separator(&mut self.writer)?;
        write!(self.writer, "{}", vd.r#type().as_ref().display_name())?;
        if let Some(default_value) = vd.default_value() {
            self.formatter.write_equals(&mut self.writer)?;
            self.serialize_value(default_value)?;
        }
        if let Some(directives) = vd.directives() {
            self.serialize_directives(directives)?;
        }
        Ok(())
    }

    fn serialize_variable_definitions<VD: VariableDefinitions>(
        &mut self,
        variable_definitions: &VD,
    ) -> fmt::Result {
        if !variable_definitions.is_empty() {
            write!(self.writer, "(")?;
            for (idx, vd) in variable_definitions.iter().enumerate() {
                if idx != 0 {
                    self.formatter.write_list_separator(&mut self.writer)?;
                }
                self.serialize_variable_definition(vd)?;
            }
            write!(self.writer, ")")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{escape_block_string, CompactFormatter, PrettyFormatter};
    use bluejay_parser::ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        Arguments, Directives, Parse, VariableValue,
    };

    macro_rules! assert_value_prints {
        ($val:literal) => {
            let parsed = VariableValue::parse($val).result.unwrap();
            let mut s = Serializer::to_string(PrettyFormatter::default());
            s.serialize_value(&parsed).unwrap();
            assert_eq!($val, s.into_inner());
        };
        ($out:literal, $in:literal) => {
            let parsed = VariableValue::parse($in).result.unwrap();
            let mut s = Serializer::to_string(PrettyFormatter::default());
            s.serialize_value(&parsed).unwrap();
            assert_eq!($out, s.into_inner());
        };
    }

    #[test]
    fn test_bool() {
        assert_value_prints!("true");
        assert_value_prints!("false");
    }

    #[test]
    fn test_enum() {
        assert_value_prints!("ONE");
    }

    #[test]
    fn test_float() {
        assert_value_prints!("1.0");
        assert_value_prints!("3.14159");
        assert_value_prints!("-1.23456");
        assert_value_prints!("10000.0", "1e4");
        assert_value_prints!("0.0");
    }

    #[test]
    fn test_int() {
        assert_value_prints!("1");
        assert_value_prints!("0");
        assert_value_prints!("-100");
    }

    #[test]
    fn test_list() {
        assert_value_prints!("[1, 2, 3]");
        assert_value_prints!("[]");
        assert_value_prints!("[[]]");
    }

    #[test]
    fn test_null() {
        assert_value_prints!("null");
    }

    #[test]
    fn test_object() {
        assert_value_prints!("{ foo: 1, bar: 2 }");
    }

    #[test]
    fn test_string() {
        assert_value_prints!(r#""""#);
        assert_value_prints!(r#""\"\\/\b\n\f\r\t""#, r#""\"\\\/\b\n\f\r\t""#);
        assert_value_prints!(r#""🔥""#);
    }

    #[test]
    fn test_variable() {
        assert_value_prints!("$foo");
    }

    #[test]
    fn test_arguments() {
        let s = "(a: 1, b: 2)";
        let parsed = Arguments::<false>::parse(s).result.unwrap();
        let mut ser = Serializer::to_string(PrettyFormatter::default());
        ser.serialize_arguments(&parsed).unwrap();
        assert_eq!(s, ser.into_inner());
    }

    #[test]
    fn test_directives() {
        let s = " @foo(a: 1, b: 2) @bar";
        let parsed = Directives::<false>::parse(s).result.unwrap();
        let mut ser = Serializer::to_string(PrettyFormatter::default());
        ser.serialize_directives(&parsed).unwrap();
        assert_eq!(s, ser.into_inner());
    }

    #[test]
    fn test_block_string_value() {
        fn write_desc(desc: &str, depth: usize) -> String {
            let mut output = String::new();
            let mut fmt = PrettyFormatter::default();
            fmt.write_description(&mut output, desc, depth).unwrap();
            output
        }

        assert_eq!("\"\"\"\n\"\"\"\n", write_desc("", 0));
        assert_eq!("    \"\"\"\n    \"\"\"\n", write_desc("", 2));
        assert_eq!(
            "\"\"\"\nThis\nis\na\nmultiline\nstring\n\"\"\"\n",
            write_desc("This\nis\na\nmultiline\nstring", 0)
        );
        assert_eq!("\"\"\"\n\\\"\"\"\n\"\"\"\n", write_desc("\"\"\"", 0));
    }

    #[test]
    fn test_schema_dump() {
        insta::glob!("definition/test_data/schema_definition/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let document: DefinitionDocument =
                DefinitionDocument::parse(input.as_str()).result.unwrap();
            let schema_definition = SchemaDefinition::try_from(&document).unwrap();
            let mut s = Serializer::to_string(PrettyFormatter::default());
            s.serialize_schema_definition(&schema_definition).unwrap();
            similar_asserts::assert_eq!(input, s.into_inner());
        });
    }

    // ── escape_block_string ─────────────────────────────────────────

    #[test]
    fn test_escape_block_string() {
        assert_eq!("hello", escape_block_string("hello"));
        assert_eq!(r#"\""""#, escape_block_string(r#"""""#));
        assert_eq!(
            r#"before \""" after"#,
            escape_block_string(r#"before """ after"#)
        );
        assert_eq!("no change", escape_block_string("no change"));
    }

    // ── CompactFormatter tests ──────────────────────────────────────

    macro_rules! assert_compact_value_prints {
        ($out:literal, $in:literal) => {
            let parsed = VariableValue::parse($in).result.unwrap();
            let mut s = Serializer::to_string(CompactFormatter);
            s.serialize_value(&parsed).unwrap();
            assert_eq!($out, s.into_inner());
        };
    }

    #[test]
    fn test_compact_value_list() {
        assert_compact_value_prints!("[1,2,3]", "[1, 2, 3]");
    }

    #[test]
    fn test_compact_value_object() {
        assert_compact_value_prints!("{foo:1,bar:2}", "{ foo: 1, bar: 2 }");
    }

    #[test]
    fn test_compact_arguments() {
        let s = "(a: 1, b: 2)";
        let parsed = Arguments::<false>::parse(s).result.unwrap();
        let mut ser = Serializer::to_string(CompactFormatter);
        ser.serialize_arguments(&parsed).unwrap();
        assert_eq!("(a:1,b:2)", ser.into_inner());
    }

    #[test]
    fn test_compact_directives() {
        let s = " @foo(a: 1, b: 2) @bar";
        let parsed = Directives::<false>::parse(s).result.unwrap();
        let mut ser = Serializer::to_string(CompactFormatter);
        ser.serialize_directives(&parsed).unwrap();
        assert_eq!(" @foo(a:1,b:2) @bar", ser.into_inner());
    }

    #[test]
    fn test_compact_schema_definition() {
        insta::glob!("definition/test_data/schema_definition/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let document: DefinitionDocument =
                DefinitionDocument::parse(input.as_str()).result.unwrap();
            let schema_definition = SchemaDefinition::try_from(&document).unwrap();
            let mut s = Serializer::to_string(CompactFormatter);
            s.serialize_schema_definition(&schema_definition).unwrap();
            let compact = s.into_inner();
            insta::assert_snapshot!(compact.clone());

            let reparsed_document: DefinitionDocument = DefinitionDocument::parse(compact.as_str())
                .result
                .unwrap_or_else(|_| {
                    panic!("compact output failed to parse for {}", path.display())
                });
            SchemaDefinition::try_from(&reparsed_document).unwrap_or_else(|_| {
                panic!(
                    "compact output failed to build schema definition for {}",
                    path.display()
                )
            });
        });
    }

    // ── PrettyFormatter with custom indent ──────────────────────────

    #[test]
    fn test_custom_indent_size() {
        let input = "type Query {\n  name: String!\n}\n";
        let document: DefinitionDocument = DefinitionDocument::parse(input).result.unwrap();
        let schema_definition = SchemaDefinition::try_from(&document).unwrap();
        let mut s = Serializer::to_string(PrettyFormatter::new(4));
        s.serialize_schema_definition(&schema_definition).unwrap();
        let output = s.into_inner();
        assert!(
            output.contains("    name"),
            "expected 4-space indent, got: {output}"
        );
    }
}
