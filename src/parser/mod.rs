pub mod recursive_descent;

use crate::ast::Node;
use crate::token::Token;

pub trait Parse<'a> {
    fn parse(&mut self, tokens: &'a [Token]) -> Result<Node, Error>;
}

#[derive(Debug)]
pub struct Error {
    pub line_number: i32,
    pub message: String,
}

impl Error {
    fn new(line_number: i32, message: &str) -> Error {
        Error {
            line_number: line_number,
            message: message.to_string(),
        }
    }
}
