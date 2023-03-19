use crate::Argument;
use paste::paste;

macro_rules! define_const_wrapper {
    ( $t:ty ) => {
        paste! {
            pub enum [<$t Wrapper>]<'a, C: $t<true>, V: $t<false>> {
                Constant(&'a C),
                Variable(&'a V),
            }
        }
    };
}

define_const_wrapper!(Argument);

#[macro_export]
macro_rules! call_const_wrapper_method {
    ( $wrapper:ident, $val:expr, $method_name:ident $(,)? ) => {
        match $val {
            $wrapper::Constant(c) => c.$method_name(),
            $wrapper::Variable(v) => v.$method_name(),
        }
    };
}
