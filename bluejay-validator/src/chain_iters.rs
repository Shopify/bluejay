#[macro_export]
macro_rules! chain_types {
    ( $first:ty, $( $rest:ty ),+ $(,)? ) => {
        std::iter::Chain<$crate::chain_types!($($rest),+), $first>
    };
    ( $t:ty ) => { $t };
}

#[macro_export]
macro_rules! chain_iters {
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        $crate::chain_iters!($($rest),+).chain($first)
    };
    ( $iter:expr ) => { $iter };
}
