#![feature(assert_matches)]

pub mod ast;
pub mod error;
mod lexical_token;
mod scanner;
mod span;

pub use error::Error;
pub use span::{HasSpan, Span};
