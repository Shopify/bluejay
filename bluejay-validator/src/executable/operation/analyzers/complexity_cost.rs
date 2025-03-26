use crate::executable::{
    operation::{Analyzer, VariableValues, Visitor},
    Cache,
};
use bluejay_core::definition::{
    FieldDefinition, ObjectTypeDefinition, OutputType, SchemaDefinition, TypeDefinition,
    TypeDefinitionReference, UnionMemberType, UnionTypeDefinition,
};
use bluejay_core::executable::{ExecutableDocument, Field};
use bluejay_core::AsIter;
use itertools::{Either, Itertools};
use std::cmp::max;
use std::collections::HashMap;

mod arena;
use arena::{Arena, NodeId};

mod cost_computer;
pub use cost_computer::{CostComputer, DefaultCostComputer, FieldMultipliers};

mod relay_cost_computer;
pub use relay_cost_computer::RelayCostComputer;

pub struct ComplexityCost<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    V: VariableValues,
    C: CostComputer<'a, E, S, V> = DefaultCostComputer,
> {
    schema_definition: &'a S,
    cost_computer: C,
    scopes_arena: Arena<ComplexityScope<'a, S::TypeDefinition, C::FieldMultipliers>>,
    scopes_stack: Vec<Option<NodeId>>,
}

impl<
        'a,
        E: ExecutableDocument,
        S: SchemaDefinition,
        V: VariableValues,
        C: CostComputer<'a, E, S, V>,
    > Visitor<'a, E, S, V> for ComplexityCost<'a, E, S, V, C>
{
    type ExtraInfo = ();
    fn new(
        operation_definition: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a V,
        _: &'a Cache<'a, E, S>,
        _: Self::ExtraInfo,
    ) -> Self {
        let mut scopes_arena = Arena::new();
        let scopes_stack = vec![Some(scopes_arena.add(ComplexityScope::default()))];
        Self {
            schema_definition,
            cost_computer: C::new(operation_definition, schema_definition, variable_values),
            scopes_arena,
            scopes_stack,
        }
    }

    fn visit_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a S::FieldDefinition,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        if !included {
            return;
        }
        let cost = self
            .cost_computer
            .cost_for_field_definition(field_definition);

        // Don't grow the costing tree for leaf fields without cost,
        // just hold their position in the traversal stack with a None scope
        if cost == 0
            && !field_definition
                .r#type()
                .base(self.schema_definition)
                .is_composite()
        {
            self.scopes_stack.push(None);
            return;
        }

        // Get the field key (custom alias or base field name)
        let field_key = field.response_name();

        // Get next insertion point in the Vec of mutable scopes, using arena tree pattern:
        // https://docs.rs/indextree/latest/indextree/#arena-based-tree-data-structure
        let next_index = self.scopes_arena.next_id();

        // Get a mutable reference to the parent scope in the traversal stack
        let parent_scope = self
            .scopes_stack
            .last()
            .copied()
            .flatten()
            .and_then(|index| self.scopes_arena.get_mut(index))
            .expect("expected a parent complexity scope");

        // Collect any multipliers that the parent scope specifies for this field
        // ie: connection > edges/nodes
        let parent_multiplier = parent_scope.multiplier_for_field(field);

        // find or create a reference to this field's scope in the parent tree of typed selections
        // ie: parent_scope.typed_selections = { Type => { "field_key" => scopes_db_index, ... } }
        let scope_index = *parent_scope
            .typed_selections
            .entry(scoped_type.name())
            .or_insert_with(|| TypedSelection {
                type_definition: scoped_type,
                inner_selection: HashMap::new(),
            })
            .inner_selection
            .entry(field_key)
            .or_insert(next_index);

        // This should really be part of a `or_insert_with` instead of the `or_insert` above,
        // but the borrow checker doesn't like that.
        if scope_index == next_index {
            let field_multipliers = self
                .cost_computer
                .field_multipliers(field_definition, field);

            self.scopes_arena.add(ComplexityScope {
                field_multipliers,
                ..Default::default()
            });
        }

        // Push the current scope reference onto the traversal stack,
        // and then get a mutable reference to the scope itself
        self.scopes_stack.push(Some(scope_index));
        let scope = self
            .scopes_arena
            .get_mut(scope_index)
            .expect("invalid complexity scope tree reference");

        // repeated scopes have a consistent argument multiplier in valid documents
        scope.multiplier = parent_multiplier;
        scope.cost = scope.cost.max(cost);
    }

    fn leave_field(
        &mut self,
        _field: &'a <E as ExecutableDocument>::Field,
        _field_definition: &'a S::FieldDefinition,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        if included {
            self.scopes_stack.pop().unwrap();
        }
    }
}

impl<
        'a,
        E: ExecutableDocument,
        S: SchemaDefinition,
        V: VariableValues,
        C: CostComputer<'a, E, S, V>,
    > Analyzer<'a, E, S, V> for ComplexityCost<'a, E, S, V, C>
{
    type Output = usize;

    fn into_output(mut self) -> Self::Output {
        self.result()
    }
}

impl<
        'a,
        E: ExecutableDocument,
        S: SchemaDefinition,
        V: VariableValues,
        C: CostComputer<'a, E, S, V>,
    > ComplexityCost<'a, E, S, V, C>
{
    fn result(&mut self) -> usize {
        let root_scope = self
            .scopes_stack
            .first()
            .copied()
            .flatten()
            .and_then(|index| self.scopes_arena.get(index))
            .unwrap();
        self.merged_max_complexity_for_scopes(&[root_scope])
    }

    fn merged_max_complexity_for_scopes(
        &self,
        scopes: &[&ComplexityScope<'a, S::TypeDefinition, C::FieldMultipliers>],
    ) -> usize {
        // build a set of all unique possible type definitions
        // with abstract types expanded to encompass all of their possible types
        let possible_type_names = scopes
            .iter()
            .flat_map(|scope| {
                scope
                    .typed_selections
                    .values()
                    .map(|typed_selection| typed_selection.type_definition)
            })
            .unique_by(|ty| ty.name())
            .flat_map(|ty| self.possible_type_names(&ty))
            .unique();

        // calculate a maximum possible cost among possible types
        possible_type_names
            .map(|possible_type_name| {
                // collect inner selections from all scopes that intersect with this possible type
                let inner_selections = scopes
                    .iter()
                    .flat_map(|scope| {
                        scope
                            .typed_selections
                            .values()
                            .filter_map(|typed_selection| {
                                self.possible_type_names(&typed_selection.type_definition)
                                    .any(|name| name == possible_type_name)
                                    .then_some(&typed_selection.inner_selection)
                            })
                    })
                    .collect::<Vec<_>>();

                self.merged_max_complexity_for_selections(inner_selections)
            })
            .max()
            .unwrap_or(0)
    }

    fn merged_max_complexity_for_selections(
        &self,
        inner_selections: Vec<&InnerSelection<'a>>,
    ) -> usize {
        // build a unique set of field keys from across inner selections.
        // the same field keys may appear in selections on different types,
        // ex: a "metafield" key may be selected on both Product and HasMetafield types.
        let unique_field_keys = inner_selections
            .iter()
            .flat_map(|child_scope| child_scope.keys())
            .unique();

        // calculate a maximum possible cost for each unique field key
        unique_field_keys
            .map(|field_key| {
                let mut base_cost = 0;
                let mut multiplier = 0;

                // collect child scopes from across composite selections
                // leaf selections report their costs directly
                let composite_scopes = inner_selections
                    .iter()
                    .filter_map(|inner_selection| {
                        inner_selection
                            .get(*field_key)
                            .and_then(|scope_index| self.scopes_arena.get(*scope_index))
                            .and_then(|child_scope| {
                                // base_cost and multiplier select their maximums from across merged scopes
                                // in case a field name has different costs in different scope types.
                                base_cost = max(base_cost, child_scope.cost);
                                multiplier = max(multiplier, child_scope.multiplier);

                                if !child_scope.typed_selections.is_empty() {
                                    Some(child_scope)
                                } else {
                                    None
                                }
                            })
                    })
                    .collect::<Vec<&ComplexityScope<'a, S::TypeDefinition, C::FieldMultipliers>>>();

                let children_cost = self.merged_max_complexity_for_scopes(&composite_scopes);

                (base_cost + children_cost) * multiplier
            })
            .sum()
    }

    fn possible_type_names(
        &self,
        ty: &TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> impl Iterator<Item = &'a str> {
        match ty {
            TypeDefinitionReference::Object(_) => Either::Left(Some(ty.name()).into_iter()),
            TypeDefinitionReference::Interface(itd) => Either::Right(Either::Left(
                self.schema_definition
                    .get_interface_implementors(itd)
                    .map(ObjectTypeDefinition::name),
            )),
            TypeDefinitionReference::Union(utd) => Either::Right(Either::Right(
                utd.union_member_types()
                    .iter()
                    .map(|union_member| union_member.name()),
            )),
            _ => Either::Left(None.into_iter()),
        }
    }
}

type InnerSelection<'a> = HashMap<&'a str, NodeId>;

struct TypedSelection<'a, T: TypeDefinition> {
    type_definition: TypeDefinitionReference<'a, T>,
    inner_selection: InnerSelection<'a>,
}

struct ComplexityScope<'a, T: TypeDefinition, F> {
    cost: usize,
    multiplier: usize,
    typed_selections: HashMap<&'a str, TypedSelection<'a, T>>,
    field_multipliers: F,
}

impl<T: TypeDefinition, F: Default> Default for ComplexityScope<'_, T, F> {
    fn default() -> Self {
        Self {
            cost: 0,
            multiplier: 1,
            typed_selections: HashMap::new(),
            field_multipliers: F::default(),
        }
    }
}

impl<T: TypeDefinition, F> ComplexityScope<'_, T, F> {
    fn multiplier_for_field<E: ExecutableDocument>(&self, field: &E::Field) -> usize
    where
        F: FieldMultipliers<E>,
    {
        self.field_multipliers.multiplier_for_field(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executable::{operation::Orchestrator, Cache};
    use bluejay_parser::ast::{
        definition::{DefaultContext, DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
        Parse,
    };
    use serde_json::Value as JsonValue;

    type ComplexityAnalyzer<'a, E, S, V> =
        Orchestrator<'a, E, S, V, ComplexityCost<'a, E, S, V, RelayCostComputer<'a, E, S, V>>>;

    const TEST_SCHEMA: &str = r#"
        directive @cost(weight: String!, kind: String) on FIELD_DEFINITION

        enum BasicEnum {
          YES
          NO
        }

        interface Node {
          id: ID!
          one: BasicObject!
        }

        interface BasicInterface {
          zeroScalar: String!
          oneObject: BasicObject!
        }

        type BasicObject implements Node & BasicInterface {
          id: ID!
          one: BasicObject!
          zeroScalar: String!
          zeroEnum: BasicEnum!
          oneObject: BasicObject!
          twoScalar: String! @cost(weight: "2.0")
          twoEnum: BasicEnum! @cost(weight: "2.0")
          twoObject: BasicObject! @cost(weight: "2.0")
        }

        union BasicUnion = BasicObject

        type Query {
          zeroScalar: String!
          zeroEnum: BasicEnum!
          oneObject: BasicObject!
          oneInterface: BasicInterface!
          oneUnion: BasicUnion!
          twoScalar: String! @cost(weight: "2.0")
          twoEnum: BasicEnum! @cost(weight: "2.0")
          fiveScalar: String! @cost(weight: "5.0")
          fiveEnum: BasicEnum! @cost(weight: "5.0")
          fiveBasicObject: BasicObject! @cost(weight: "5.0")

          node(id: ID!): Node

          zeroScalarList: [String!]!
          zeroEnumList: [BasicEnum!]!
          oneObjectList: [BasicObject!]!
          fiveObjectList: [BasicObject!]! @cost(weight: "5.0")

          oneObjectConnection(first: Int!, last: Int!): BasicObjectConnection @cost(weight: "1.0", kind: "connection")
          twoObjectConnection(first: Int!, last: Int!): BasicObjectConnection @cost(weight: "2.0", kind: "connection")
        }

        type PageInfo {
            hasNextPage: Boolean!
            hasPreviousPage: Boolean!
        }

        type BasicObjectEdge {
            cursor: String!
            node: BasicObject!
        }

        type BasicObjectConnection {
            edges: [BasicObjectEdge!]! @cost(weight: "0.0")
            nodes: [BasicObject!]!
            pageInfo: PageInfo! @cost(weight: "0.0")
        }

        type Comment {
            body: String!
        }

        interface HasComments {
          comments: [Comment]!
        }

        type Product implements Node & HasComments {
            id: ID!
            one: BasicObject!
            comments: [Comment]!
        }

        type User implements Node & HasComments {
            id: ID!
            one: BasicObject!
            comments: [Comment]!
            oneObject: BasicObject!
        }

        schema {
          query: Query
        }
    "#;

    fn check_complexity_with_operation_name_and_variables(
        source: &str,
        operation_name: Option<&str>,
        variables: &JsonValue,
        expected_complexity: usize,
    ) {
        let definition_document: DefinitionDocument<'_, DefaultContext> =
            DefinitionDocument::parse(TEST_SCHEMA).expect("Schema had parse errors");
        let schema_definition =
            SchemaDefinition::try_from(&definition_document).expect("Schema had errors");
        let executable_document = ExecutableDocument::parse(source)
            .unwrap_or_else(|_| panic!("Document had parse errors"));
        let cache = Cache::new(&executable_document, &schema_definition);
        let variables = variables.as_object().expect("Variables must be an object");
        let complexity = ComplexityAnalyzer::analyze(
            &executable_document,
            &schema_definition,
            operation_name,
            variables,
            &cache,
            (),
        )
        .unwrap();

        assert_eq!(complexity, expected_complexity);
    }

    fn check_complexity_with_operation_name(
        source: &str,
        operation_name: Option<&str>,
        expected_complexity: usize,
    ) {
        check_complexity_with_operation_name_and_variables(
            source,
            operation_name,
            &serde_json::json!({}),
            expected_complexity,
        )
    }

    fn check_complexity_with_variables(
        source: &str,
        variables: JsonValue,
        expected_complexity: usize,
    ) {
        check_complexity_with_operation_name_and_variables(
            source,
            None,
            &variables,
            expected_complexity,
        )
    }

    fn check_complexity(source: &str, expected_complexity: usize) {
        check_complexity_with_operation_name_and_variables(
            source,
            None,
            &serde_json::json!({}),
            expected_complexity,
        )
    }

    #[test]
    fn basic_cost_metrics() {
        check_complexity(r#"{ zeroScalar }"#, 0);
        check_complexity(r#"{ zeroEnum }"#, 0);
        check_complexity(r#"{ oneObject { zeroScalar } }"#, 1);
        check_complexity(r#"{ oneInterface { zeroScalar } }"#, 1);
        check_complexity(r#"{ oneUnion { ...on BasicObject { zeroScalar } } }"#, 1);
    }

    #[test]
    fn basic_list_cost_metrics() {
        check_complexity(r#"{ zeroScalarList }"#, 0);
        check_complexity(r#"{ zeroEnumList }"#, 0);
        check_complexity(r#"{ oneObjectList { zeroScalar } }"#, 1);
        check_complexity(r#"{ fiveObjectList { zeroScalar } }"#, 5);
    }

    #[test]
    fn basic_cost_metrics_nested() {
        check_complexity(
            r#"{
          oneObject { # 1 + 4 = 5
            oneObject { oneObject { zeroEnum twoScalar } } # 1 + 1 + 0 + 2 = 4
          }
        }"#,
            5,
        );
    }

    #[test]
    fn field_cost_metrics() {
        check_complexity(r#"{ fiveScalar }"#, 5);
        check_complexity(r#"{ fiveEnum }"#, 5);
        check_complexity(r#"{ fiveBasicObject { zeroScalar } }"#, 5);
    }

    #[test]
    fn field_cost_metrics_nested() {
        check_complexity(
            r#"query { # 5 + 5 + 9 = 19
          fiveScalar # 5
          fiveEnum # 5
          fiveBasicObject { # 5 + 4 = 9
            twoObject { # 2 + 2 = 4
              twoScalar
            }
          }
        }"#,
            19,
        );
    }

    #[test]
    fn active_operation_name() {
        check_complexity_with_operation_name(
            r#"
          query { fiveScalar }
        "#,
            None,
            5,
        );

        check_complexity_with_operation_name(
            r#"
          query Test { fiveScalar }
        "#,
            None,
            5,
        );

        check_complexity_with_operation_name(
            r#"
            query Test1 { twoScalar }
            query Test2 { fiveScalar }
        "#,
            Some("Test1"),
            2,
        );
    }

    #[test]
    fn gracefully_handles_invalid_fields_and_fragment_types() {
        // final reported result is None based on invalid query status
        check_complexity(
            r#"{
            bogusField
            ...on BogusType { bogusField }
            ...BogusSpread
            fiveScalar
        }"#,
            5,
        );
    }

    #[test]
    fn fragment_definitions() {
        check_complexity(
            r#"
          query { # 3 + 7 = 10
            oneObject { ...Attrs } # 1 + 2 = 3
            fiveBasicObject { ...Attrs } # 5 + 2 = 7
          }
          fragment Attrs on BasicObject {
            twoObject { zeroScalar } # 2 + 0 = 2
          }
        "#,
            10,
        );
    }

    #[test]
    fn skip_and_include_fields_with_bool_literals() {
        check_complexity(
            r#"query { # 1 + 7 = 8
            oneObject { zeroScalar } # 1 + 0 = 1
            fiveBasicObject @skip(if: false) { twoScalar } # (5 + 2) * 1 = 7
        }"#,
            8,
        );

        check_complexity(
            r#"query { # 1 + 0 = 1
            oneObject { zeroScalar } # 1 + 0 = 1
            fiveBasicObject @skip(if: true) { twoScalar } # (5 + 0) * 0 = 0
        }"#,
            1,
        );

        check_complexity(
            r#"query {
            oneObject { zeroScalar }
            fiveBasicObject @include(if: false) { twoScalar }
        }"#,
            1,
        );

        check_complexity(
            r#"query {
            oneObject { zeroScalar }
            fiveBasicObject @include(if: true) { twoScalar }
        }"#,
            8,
        );
    }

    #[test]
    fn skip_and_include_fields_with_variables() {
        check_complexity_with_variables(
            r#"query($enabled: Boolean) {
            oneObject { zeroScalar }
            fiveBasicObject @skip(if: $enabled) { twoScalar }
        }"#,
            serde_json::json!({ "enabled": false }),
            8,
        );

        check_complexity_with_variables(
            r#"query($enabled: Boolean) {
            oneObject { zeroScalar }
            fiveBasicObject @skip(if: $enabled) { twoScalar }
        }"#,
            serde_json::json!({ "enabled": true }),
            1,
        );

        check_complexity_with_variables(
            r#"query($enabled: Boolean) {
            oneObject { zeroScalar }
            fiveBasicObject @include(if: $enabled) { twoScalar }
        }"#,
            serde_json::json!({ "enabled": false }),
            1,
        );

        check_complexity_with_variables(
            r#"query($enabled: Boolean) {
            oneObject { zeroScalar }
            fiveBasicObject @include(if: $enabled) { twoScalar }
        }"#,
            serde_json::json!({ "enabled": true }),
            8,
        );
    }

    #[test]
    fn skip_and_include_fields_with_default_variables() {
        check_complexity(
            r#"query($enabled: Boolean = false) {
            oneObject { zeroScalar }
            fiveBasicObject @skip(if: $enabled) { twoScalar }
        }"#,
            8,
        );

        check_complexity(
            r#"query($enabled: Boolean = true) {
            oneObject { zeroScalar }
            fiveBasicObject @skip(if: $enabled) { twoScalar }
        }"#,
            1,
        );
    }

    #[test]
    fn skip_and_include_inline_fragments_with_bool_literals() {
        check_complexity(
            r#"query {
            oneObject { zeroScalar }
            ... @skip(if: false) { fiveBasicObject { twoScalar } }
        }"#,
            8,
        );

        check_complexity(
            r#"query {
            oneObject { zeroScalar }
            ... @skip(if: true) { fiveBasicObject { twoScalar } }
        }"#,
            1,
        );

        check_complexity(
            r#"query {
            oneObject { zeroScalar }
            ... @include(if: false) { fiveBasicObject { twoScalar } }
        }"#,
            1,
        );

        check_complexity(
            r#"query {
            oneObject { zeroScalar }
            ... @include(if: true) { fiveBasicObject { twoScalar } }
        }"#,
            8,
        );
    }

    #[test]
    fn skip_and_include_fragment_spreads_with_bool_literals() {
        check_complexity(
            r#"
            fragment Stuff on Query { fiveBasicObject { twoScalar } }
            query {
                oneObject { zeroScalar }
                ... Stuff @skip(if: false)
            }
        "#,
            8,
        );

        check_complexity(
            r#"
            fragment Stuff on Query { fiveBasicObject { twoScalar } }
            query {
                oneObject { zeroScalar }
                ... Stuff @skip(if: true)
            }
        "#,
            1,
        );

        check_complexity(
            r#"
            fragment Stuff on Query { fiveBasicObject { twoScalar } }
            query {
                oneObject { zeroScalar }
                ... Stuff @include(if: false)
            }
        "#,
            1,
        );

        check_complexity(
            r#"
            fragment Stuff on Query { fiveBasicObject { twoScalar } }
            query {
                oneObject { zeroScalar }
                ... Stuff @include(if: true)
            }
        "#,
            8,
        );
    }

    #[test]
    fn skip_and_include_fragments_with_variables() {
        check_complexity_with_variables(
            r#"query($enabled: Boolean) {
            oneObject { zeroScalar }
            ... @skip(if: $enabled) { fiveBasicObject { twoScalar } }
        }"#,
            serde_json::json!({ "enabled": false }),
            8,
        );

        check_complexity_with_variables(
            r#"query($enabled: Boolean) {
            oneObject { zeroScalar }
            ... @skip(if: $enabled) { fiveBasicObject { twoScalar } }
        }"#,
            serde_json::json!({ "enabled": true }),
            1,
        );

        check_complexity_with_variables(
            r#"
            fragment Stuff on Query { fiveBasicObject { twoScalar } }
            query($enabled: Boolean) {
                oneObject { zeroScalar }
                ... Stuff @include(if: $enabled)
            }
        "#,
            serde_json::json!({ "enabled": false }),
            1,
        );

        check_complexity_with_variables(
            r#"
            fragment Stuff on Query { fiveBasicObject { twoScalar } }
            query($enabled: Boolean) {
                oneObject { zeroScalar }
                ... Stuff @include(if: $enabled)
            }
        "#,
            serde_json::json!({ "enabled": true }),
            8,
        );
    }

    #[test]
    fn skip_and_include_fragments_with_default_variables() {
        check_complexity(
            r#"query($enabled: Boolean = false) {
            oneObject { zeroScalar }
            ... @skip(if: $enabled) { fiveBasicObject { twoScalar } }
        }"#,
            8,
        );

        check_complexity(
            r#"query($enabled: Boolean = true) {
            oneObject { zeroScalar }
            ... @skip(if: $enabled) { fiveBasicObject { twoScalar } }
        }"#,
            1,
        );
    }

    #[test]
    fn skipped_paths_still_cost_when_revisited() {
        check_complexity(
            r#"{
            oneObjectConnection(first: 7) { # 1 + 0 = 1
              edges @skip(if: true) { node { twoScalar } } # skip = 0
            }
            oneObjectConnection(first: 7) { # 0 + (3 * floor(2 * log(7))) = 9
              edges { node { twoScalar } } # (0 + 1 + 2) = 3
            }
        }"#,
            10,
        );
    }

    #[test]
    fn connection_with_slicing_arguments_and_sized_fields() {
        check_complexity(
            r#"{
            oneObjectConnection(first: 7) { # 1 + (3 + 3) * floor(2 * log(7))) = 19
              edges { node { zeroScalar twoScalar } } # (0 + 1 + 0 + 2) = 3
              nodes { zeroScalar twoScalar } # (1 + 0 + 2) = 3
              pageInfo { hasNextPage } # 0
            }
        }"#,
            19,
        );
    }

    #[test]
    fn connection_with_slicing_arguments_using_variables() {
        check_complexity_with_variables(
            r#"query($first: Int) {
            oneObjectConnection(first: $first) { # 1 + (3 + 3) * floor(2 * log(7))) = 19
              edges { node { zeroScalar twoScalar } } # (0 + 1 + 0 + 2) = 3
              nodes { zeroScalar twoScalar } # (1 + 0 + 2) = 3
              pageInfo { hasNextPage } # 0
            }
        }"#,
            serde_json::json!({ "first": 7 }),
            19,
        );
    }

    #[test]
    fn connection_with_slicing_arguments_using_default_variables() {
        check_complexity(
            r#"query($first: Int = 7) {
            oneObjectConnection(first: $first) { # 1 + (3 + 3) * floor(2 * log(7))) = 19
              edges { node { zeroScalar twoScalar } } # (0 + 1 + 0 + 2) = 3
              nodes { zeroScalar twoScalar } # (1 + 0 + 2) = 3
              pageInfo { hasNextPage } # 0
            }
        }"#,
            19,
        );
    }

    #[test]
    fn connection_with_multiple_slicing_arguments_uses_max() {
        check_complexity(
            r#"query($last: Int = 0, $first: Int = 7) {
            oneObjectConnection(last: $last, first: $first) { # 1 + 3 * floor(2 * log(7))) = 10
              edges { node { twoScalar } } # (0 + 1 + 2) = 3
            }
        }"#,
            10,
        );

        check_complexity(
            r#"query($first: Int = 7, $last: Int) {
            oneObjectConnection(first: $first, last: $last) { # 1 + 3 * floor(2 * log(7))) = 10
              edges { node { twoScalar } } # (0 + 1 + 2) = 3
            }
        }"#,
            10,
        );

        check_complexity(
            r#"query($first: Int = 7) {
            oneObjectConnection(first: $first, last: null) { # 1 + 3 * floor(2 * log(7))) = 10
              edges { node { twoScalar } } # (0 + 1 + 2) = 3
            }
        }"#,
            10,
        );
    }

    #[test]
    fn connection_with_slicing_arguments_and_sized_fields_via_inline_fragment() {
        check_complexity(
            r#"
            query {
                oneObjectConnection(first: 7) { # 1 + (3 + 3) * floor(2 * log(7))) = 19
                  ...on BasicObjectConnection {
                    edges { node { zeroScalar twoScalar } } # (1 + 0 + 0 + 2) = 3
                    ...on BasicObjectConnection {
                        nodes { zeroScalar twoScalar } # (1 + 0 + 2) = 3
                    }
                  }
                  pageInfo { hasNextPage } # 0
                }
            }
        "#,
            19,
        );
    }

    #[test]
    fn connection_with_slicing_arguments_and_sized_fields_via_fragment_spread() {
        check_complexity(
            r#"
            query { # 19 + 13 = 32
                seven: oneObjectConnection(first: 7) { # 1 + (3 + 3) * floor(2 * log(7))) = 19
                  ...ConnectionAttrs
                  pageInfo { hasNextPage } # 0
                }
                three: oneObjectConnection(first: 3) { # 1 + (3 + 3) * floor(2 * log(3))) = 13
                  ...ConnectionAttrs
                  pageInfo { hasNextPage } # 0
                }
            }
            fragment ConnectionAttrs on BasicObjectConnection {
                edges { node { zeroScalar twoScalar } } # (0 + 1 + 0 + 2) = 3
                ...ConnectionNodeAttrs
            }
            fragment ConnectionNodeAttrs on BasicObjectConnection {
                nodes { zeroScalar twoScalar } # (1 + 0 + 2) = 3
            }
        "#,
            32,
        );
    }

    #[test]
    fn zero_and_negative_multipliers_are_zero() {
        check_complexity(
            r#"{
            oneObjectConnection(first: 0) { # 1 + (3 * 0) = 1
              edges { node { twoScalar } } # (0 + 1 + 2) = 3
            }
        }"#,
            1,
        );

        check_complexity(
            r#"{
            oneObjectConnection(first: -7) { # 1 + (3 * 0) = 1
              edges { node { twoScalar } } # (0 + 1 + 2) = 3
            }
        }"#,
            1,
        );
    }

    #[test]
    fn connection_with_base_cost() {
        check_complexity(
            r#"{
            twoObjectConnection(first: 7) { # 2 + 3 * floor(2 * log(7))) = 11
              edges { node { zeroScalar twoScalar } } # (0 + 1 + 0 + 2) = 3
            }
        }"#,
            11,
        );
    }

    #[test]
    fn connection_with_skipped_sized_fields() {
        check_complexity(
            r#"{
            oneObjectConnection(first: 7) { # 1 + 3 * floor(2 * log(7))) = 10
              edges @skip(if: true) { node { zeroScalar twoScalar } } # (0 + 1 + 0 + 2) * 0 = 0
              nodes { zeroScalar twoScalar } # (1 + 0 + 2) = 3
              pageInfo { hasNextPage } # 0
            }
        }"#,
            10,
        );
    }

    #[test]
    fn basic_overlapping_field_paths_only_cost_once() {
        check_complexity(
            r#"{ # 3 + 2 = 5
            oneObject { twoScalar } # 1 + 2 = 3
            oneObject { twoEnum } # 0 + 2 = 2
        }"#,
            5,
        );
    }

    #[test]
    fn overlapping_field_paths_with_multipliers_only_cost_once() {
        check_complexity(
            r#"{
            twoObjectConnection(first: 7) { # 2 + (3 + 2 + 3) * floor(2 * log(7))) = 26
              ...EdgesOnly
              ...EdgesAndNodes
            }
        }
        fragment EdgesOnly on BasicObjectConnection {
          edges { node { twoScalar } } # 0 + 1 + 2 = 3
        }
        fragment EdgesAndNodes on BasicObjectConnection {
          edges { node { twoScalar twoEnum } } # X + X + X + 2 = 2
          nodes { twoScalar } # 1 + 2 = 3
        }"#,
            26,
        );
    }

    #[test]
    fn performs_inline_traversal_of_fragment_spreads() {
        check_complexity(
            r#"{
            node(id: "1") { # 1 + max(1, 3) = 4
                ...OnAbstract
                ...OnConcrete
            }
        }
        fragment OnAbstract on BasicInterface { # 1
          oneObject { zeroScalar } # 1
        }
        fragment OnConcrete on BasicObject { # 2 + 1 from BasicInterface = 3
          twoObject { zeroScalar } # 2
        }"#,
            4,
        );
    }

    #[test]
    fn abstract_scope_uses_max_fragment_cost() {
        check_complexity(
            r#"{
          node(id: "r2d2c3p0") { # 1 + max(1, 4, 2, 1) = 5
            id
            ...on Node { # 1
              one { zeroScalar }
            }
            ...on Product { # 2 + HasComments = 3
              featuredImage: one { zeroScalar }
              featuredMedia: one { zeroScalar }
            }
            ...on User { # 1 + HasComments = 2
              companyContactProfiles: one { zeroScalar }
            }
            ...on HasComments { # 1 = 1
              comments { body }
            }
          }
        }"#,
            5,
        );
    }

    #[test]
    fn nested_abstract_scopes_merge_possible_costs() {
        check_complexity(
            r#"{
          node(id: "r2d2c3p0") { # 1 + max(3, 3) = 4
            ... {
              ...on Product {
                product1: one { zeroScalar }
                product2: one { zeroScalar }
              }
              ...on User {
                user1: one { zeroScalar }
              }
            }
            ...on Product {
              product3: one { zeroScalar }
            }
            ...on User {
              user2: one { zeroScalar }
              user3: one { zeroScalar }
            }
          }
        }"#,
            4,
        );
    }

    #[test]
    fn overlapping_abstract_scopes_merge_possible_costs() {
        check_complexity(
            r#"{
          node(id: "r2d2c3p0") { # 1 + max(3, 2, 2) = 4
            ...on Product { # 1 + 1 + 1 = 3
              product1: one { zeroScalar } # 1
              product2: one { zeroScalar } # 1
            }
            ...on User { # 1 + 1 = 2
              user2: one { zeroScalar } # 1
              user3: one { zeroScalar } # 1
            }
          }
          node(id: "r2d2c3p0") { # overlapping scope
            ...on Product { # overlapping scope
              product2: one { zeroScalar } # 0
              product3: one { zeroScalar } # 1
            }
            ...on BasicObject { # 1 + 1 = 2
              basic1: one { zeroScalar } # 1
              basic2: one { zeroScalar } # 1
            }
          }
        }"#,
            4,
        );
    }

    #[test]
    fn does_not_traverse_recursive_fragment_cycles() {
        check_complexity(
            r#"
          query {
            node(id: "r2d2c3p0") { ...Alpha }
          }
          fragment Alpha on Product {
            a: one { zeroScalar }
            ...Bravo
          }
          fragment Bravo on Product {
            b: one { zeroScalar }
            ...Alpha
          }
        "#,
            3,
        );
    }

    #[test]
    fn skips_valid_typed_selections_under_invalid_paths() {
        check_complexity(
            r#"
          query {
            validScope: oneObject {
              invalidScope {
                ...on Product {
                  validScope: one { id }
                }
              }
            }
          }
        "#,
            1,
        );
    }
}
