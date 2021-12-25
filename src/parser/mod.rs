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

// TODO: Create a macro that will produce something like the following:
// test_parser!(Parser::new())
//
//
// #[cfg(test)]
// mod tests {
//     use crate::ast::{
//         ComparisonOperator, EqualityOperator, Literal, Node, ProductOperator, SumOperator,
//         UnaryOperator,
//     };
//     use crate::parser::Parse;
//     use crate::token::{Token, TokenType};

//     struct TestCase<'a> {
//         input: &'a [Token<'a>],
//         expected_output: Node,
//     }

//     #[test]
//     fn it_works() {
//         let output = Parser::new();
//         assert_eq!(2 + 2, 4);
//     }
// }
