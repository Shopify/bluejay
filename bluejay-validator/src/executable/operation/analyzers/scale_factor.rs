use crate::executable::{
    operation::{Analyzer, Visitor},
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

type InnerSelection<'a> = HashMap<&'a str, NodeId>;

struct TypedSelection<'a, T: TypeDefinition> {
    type_definition: TypeDefinitionReference<'a, T>,
    inner_selection: InnerSelection<'a>,
}

mod scale_factor_computer;
pub use scale_factor_computer::{DefaultScaleFactorComputer, ScaleFactorComputer};
struct ScaleFactorScope<'a, T: TypeDefinition, F> {
    cost: usize,
    multiplier: usize,
    typed_selections: HashMap<&'a str, TypedSelection<'a, T>>,
    field_multipliers: F,
}

pub struct ScaleFactorCost<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    C: ScaleFactorComputer<'a, E, S> = DefaultScaleFactorComputer,
> {
    schema_definition: &'a S,
    scale_factor_computer: C,
    scopes_arena: Arena<ScaleFactorScope<'a, S::TypeDefinition, C::FieldMultipliers>>,
    scopes_stack: Vec<Option<NodeId>>,
}

// todo - implement logic to calculate scale factor
