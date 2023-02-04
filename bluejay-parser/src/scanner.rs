use crate::lexical_token::LexicalToken;
use crate::Span;
mod logos_scanner;
mod scan_error;
pub use logos_scanner::LogosScanner;
pub use scan_error::ScanError;

pub trait Scanner<'a>: Iterator<Item = Result<LexicalToken<'a>, ScanError>> {
    fn empty_span(&self) -> Span;
}
