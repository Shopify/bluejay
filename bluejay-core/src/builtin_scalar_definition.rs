use strum::{AsRefStr, Display, EnumIter, EnumString, IntoStaticStr, VariantNames};

#[derive(
    IntoStaticStr,
    AsRefStr,
    VariantNames,
    EnumString,
    EnumIter,
    Display,
    Clone,
    Copy,
    Debug,
    PartialEq,
)]
pub enum BuiltinScalarDefinition {
    Int,
    Float,
    String,
    Boolean,
    ID,
}

impl BuiltinScalarDefinition {
    pub fn name(&self) -> &'static str {
        self.into()
    }
}
