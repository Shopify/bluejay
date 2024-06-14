use crate::{Context, MergedValue};
use bluejay_core::{executable::ExecutableDocument, Argument, Arguments, AsIter};

pub struct MergedArgument<'a, const CONST: bool> {
    name: &'a str,
    value: MergedValue<'a, CONST>,
}

impl<'a, const CONST: bool> Argument<CONST> for MergedArgument<'a, CONST> {
    type Value = MergedValue<'a, CONST>;

    fn name(&self) -> &str {
        self.name
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

pub struct MergedArguments<'a, const CONST: bool> {
    arguments: Vec<MergedArgument<'a, CONST>>,
}

impl<'a, const CONST: bool> Arguments<CONST> for MergedArguments<'a, CONST> {
    type Argument = MergedArgument<'a, CONST>;
}

impl<'a, const CONST: bool> AsIter for MergedArguments<'a, CONST> {
    type Item = MergedArgument<'a, CONST>;
    type Iterator<'b> = std::slice::Iter<'b, MergedArgument<'a, CONST>> where Self: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.arguments.iter()
    }
}

impl<'a, const CONST: bool> MergedArguments<'a, CONST> {
    pub(crate) fn new<E: ExecutableDocument>(
        arguments: &'a E::Arguments<CONST>,
        context: &Context<'a, E>,
    ) -> Self {
        let arguments = arguments
            .iter()
            .map(|argument| MergedArgument {
                name: argument.name(),
                value: MergedValue::new(argument.value(), context),
            })
            .collect();
        Self { arguments }
    }
}
