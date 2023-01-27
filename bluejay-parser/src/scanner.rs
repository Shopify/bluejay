use crate::lexical_token::LexicalToken;
use crate::Span;
pub mod logos_scanner;
mod scan_error;
pub use scan_error::ScanError;

pub trait Scanner<'a>: Iterator<Item=Result<LexicalToken<'a>, ScanError>> {
    fn empty_span(&self) -> Span;
}
