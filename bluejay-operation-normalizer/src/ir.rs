use bumpalo::collections::Vec as BVec;

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

pub(crate) struct NormalizedField<'a, 'bump> {
    pub name: &'a str,
    pub arg_names: BVec<'bump, &'a str>,
    pub directives: BVec<'bump, NormalizedDirective<'a, 'bump>>,
    pub selections: BVec<'bump, NormalizedSelection<'a, 'bump>>,
}

pub(crate) struct NormalizedInlineFragment<'a, 'bump> {
    pub type_condition: Option<&'a str>,
    pub directives: BVec<'bump, NormalizedDirective<'a, 'bump>>,
    pub selections: BVec<'bump, NormalizedSelection<'a, 'bump>>,
}

pub(crate) enum NormalizedSelection<'a, 'bump> {
    Field(NormalizedField<'a, 'bump>),
    InlineFragment(NormalizedInlineFragment<'a, 'bump>),
}
