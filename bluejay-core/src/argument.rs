use crate::{AsIter, Value};

pub trait Argument<const CONST: bool> {
    type Value: Value<CONST>;

    fn name(&self) -> &str;
    fn value(&self) -> &Self::Value;
}

pub trait ConstArgument: Argument<true> {}
pub trait VariableArgument: Argument<false> {}

impl<T: Argument<true>> ConstArgument for T {}
impl<T: Argument<false>> VariableArgument for T {}

pub trait Arguments<const CONST: bool>: AsIter<Item = Self::Argument> {
    type Argument: Argument<CONST>;

    #[inline]
    fn equivalent(optional_self: Option<&Self>, optional_other: Option<&Self>) -> bool {
        match (optional_self, optional_other) {
            (None, None) => true,
            (None, Some(other)) => other.is_empty(),
            (Some(s), None) => s.is_empty(),
            (Some(s), Some(other)) => {
                // For small argument lists, use O(n*m) comparison which avoids HashMap allocation
                let s_count = s.len();
                let o_count = other.len();
                if s_count != o_count {
                    return false;
                }
                // Every arg in self must have a matching arg in other (same name, same value)
                s.iter().all(|s_arg| {
                    other.iter().any(|o_arg| {
                        s_arg.name() == o_arg.name()
                            && s_arg.value().as_ref() == o_arg.value().as_ref()
                    })
                })
            }
        }
    }
}

pub trait ConstArguments: Arguments<true> {}
pub trait VariableArguments: Arguments<false> {}

impl<T: Arguments<true>> ConstArguments for T {}
impl<T: Arguments<false>> VariableArguments for T {}
