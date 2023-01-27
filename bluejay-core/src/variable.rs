pub trait Variable {
    fn name(&self) -> &str;
}

impl Variable for String {
    fn name(&self) -> &str {
        self.as_str()
    }
}
