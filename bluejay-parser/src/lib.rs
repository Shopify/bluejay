#![feature(iter_intersperse)]

mod scanner;
mod lexical_token;
mod span;
mod error;
pub mod ast;

pub use scanner::Scanner;
pub use scanner::logos_scanner::LogosScanner;
pub use span::Span;
pub use ast::parse;
pub use error::Error;
