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
