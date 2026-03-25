#[derive(Clone, Debug)]
pub(crate) struct NormalizedDirective<'a> {
    pub name: &'a str,
    /// Sorted argument names. Values are irrelevant (all become `$_`).
    /// Used for both serialization and directive equality during IF merging.
    pub arg_names: Vec<&'a str>,
}

impl PartialEq for NormalizedDirective<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arg_names == other.arg_names
    }
}

impl Eq for NormalizedDirective<'_> {}

impl PartialOrd for NormalizedDirective<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NormalizedDirective<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .cmp(other.name)
            .then_with(|| self.arg_names.cmp(&other.arg_names))
    }
}

pub(crate) struct NormalizedField<'a> {
    pub name: &'a str,
    /// Sorted argument names. All values serialize as `$_`.
    pub arg_names: Vec<&'a str>,
    pub directives: Vec<NormalizedDirective<'a>>,
    pub selections: Vec<NormalizedSelection<'a>>,
}

pub(crate) struct NormalizedInlineFragment<'a> {
    pub type_condition: Option<&'a str>,
    pub directives: Vec<NormalizedDirective<'a>>,
    pub selections: Vec<NormalizedSelection<'a>>,
}

pub(crate) enum NormalizedSelection<'a> {
    Field(NormalizedField<'a>),
    InlineFragment(NormalizedInlineFragment<'a>),
}
