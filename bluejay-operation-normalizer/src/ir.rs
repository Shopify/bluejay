//! Normalized IR types used between the build and serialize phases.
//!
//! These represent the output of algorithm step 2: a tree of fields and inline fragments
//! with aliases removed, fragments expanded, values erased, and everything sorted.
//! All collections use bump-allocated vectors ([`BVec`]) for arena allocation.

use bumpalo::collections::Vec as BVec;

/// A directive with its name and sorted argument names (step 2a).
/// Argument values are omitted during serialization.
#[derive(Clone, Debug)]
pub(crate) struct NormalizedDirective<'a, 'bump> {
    pub name: &'a str,
    pub arg_names: BVec<'bump, &'a str>,
}

impl PartialEq for NormalizedDirective<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arg_names == other.arg_names
    }
}

impl Eq for NormalizedDirective<'_, '_> {}

impl PartialOrd for NormalizedDirective<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NormalizedDirective<'_, '_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .cmp(other.name)
            .then_with(|| self.arg_names.as_slice().cmp(other.arg_names.as_slice()))
    }
}

/// A field with alias removed, arguments and directives sorted, and child
/// selections recursively normalized (step 2a).
pub(crate) struct NormalizedField<'a, 'bump> {
    /// The underlying field name (alias is dropped).
    pub name: &'a str,
    /// Argument names sorted alphabetically. Values are erased.
    pub arg_names: BVec<'bump, &'a str>,
    /// Directives sorted by name, then by argument names.
    pub directives: BVec<'bump, NormalizedDirective<'a, 'bump>>,
    /// Recursively normalized child selections.
    pub selections: BVec<'bump, NormalizedSelection<'a, 'bump>>,
}

/// An inline fragment produced by expanding a named fragment spread (step 2b)
/// or kept from an existing inline fragment (step 2c). Inline fragments with
/// matching `(type_condition, directives)` are merged (step 2d).
pub(crate) struct NormalizedInlineFragment<'a, 'bump> {
    pub type_condition: Option<&'a str>,
    pub directives: BVec<'bump, NormalizedDirective<'a, 'bump>>,
    pub selections: BVec<'bump, NormalizedSelection<'a, 'bump>>,
}

/// A normalized selection: either a field or an inline fragment.
/// Sorted with fields first (by name), then inline fragments (by type condition,
/// then directives) — see step 2e.
pub(crate) enum NormalizedSelection<'a, 'bump> {
    Field(NormalizedField<'a, 'bump>),
    InlineFragment(NormalizedInlineFragment<'a, 'bump>),
}
