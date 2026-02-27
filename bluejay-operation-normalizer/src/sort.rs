#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SelectionSortKey<'a> {
    Field(&'a str),
    FragmentSpread(&'a str),
    InlineFragment(Option<&'a str>),
}
