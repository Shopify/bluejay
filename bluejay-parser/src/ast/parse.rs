use crate::ast::{FromTokens, ScannerTokens};
use crate::scanner::LogosScanner;
use crate::Error;

pub trait Parse<'a>: FromTokens<'a> {
    fn parse(s: &'a str) -> Result<Self, Vec<Error>>;
}

impl<'a, T: FromTokens<'a>> Parse<'a> for T {
    fn parse(s: &'a str) -> Result<Self, Vec<Error>> {
        let scanner = LogosScanner::new(s);
        let mut tokens = ScannerTokens::new(scanner);

        let result = T::from_tokens(&mut tokens);

        if tokens.errors.is_empty() {
            result.map_err(|err| vec![err.into()])
        } else {
            Err(tokens.errors.into_iter().map(Into::into).collect())
        }
    }
}
