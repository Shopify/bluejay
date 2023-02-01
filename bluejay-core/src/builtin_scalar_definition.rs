use crate::definition::ScalarTypeDefinition;
use strum::{AsRefStr, Display, EnumIter, EnumString, EnumVariantNames, IntoStaticStr};

#[derive(
    IntoStaticStr, AsRefStr, EnumVariantNames, EnumString, EnumIter, Display, Clone, Copy, Debug,
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

impl ScalarTypeDefinition for BuiltinScalarDefinition {
    fn description(&self) -> Option<&str> {
        None
    }

    fn name(&self) -> &str {
        self.into()
    }
}
