use super::{Error, Parse};
use crate::ast::{
    ComparisonOperator, EqualityOperator, Literal, Node, ProductOperator, SumOperator,
    UnaryOperator,
};
use crate::token::{Token, TokenType};

pub struct RecursiveDescentParser<'a> {
    cursor: usize,
    tokens: &'a [Token<'a>],
}

impl RecursiveDescentParser<'_> {
    pub fn new() -> Self {
        RecursiveDescentParser {
            cursor: 0,
            tokens: &[],
        }
    }

    fn parse_expression(&mut self) -> Result<Node, Error> {
        let node = Node::Expression(Box::new(self.parse_series()?));
        Ok(node)
    }

    fn parse_series(&mut self) -> Result<Node, Error> {
        let mut node = self.parse_equality()?;

        while self.cursor < self.tokens.len() {
            match self.tokens[self.cursor].token_type {
                TokenType::Comma => {
                    self.cursor += 1;
                    node = Node::Series(Box::new(node), Box::new(self.parse_equality()?));
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_equality(&mut self) -> Result<Node, Error> {
        let mut node = self.parse_comparison()?;

        while self.cursor < self.tokens.len() {
            match self.tokens[self.cursor].token_type {
                TokenType::EqualEqual => {
                    self.cursor += 1;
                    node = Node::Equality(
                        EqualityOperator::Equal,
                        Box::new(node),
                        Box::new(self.parse_comparison()?),
                    );
                }
                TokenType::BangEqual => {
                    self.cursor += 1;
                    node = Node::Equality(
                        EqualityOperator::NotEqual,
                        Box::new(node),
                        Box::new(self.parse_comparison()?),
                    );
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<Node, Error> {
        let mut node = self.parse_sum()?;

        while self.cursor < self.tokens.len() {
            match self.tokens[self.cursor].token_type {
                TokenType::Greater => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        ComparisonOperator::Greater,
                        Box::new(node),
                        Box::new(self.parse_sum()?),
                    )
                }
                TokenType::GreaterEqual => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        ComparisonOperator::GreaterEqual,
                        Box::new(node),
                        Box::new(self.parse_sum()?),
                    )
                }
                TokenType::Less => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        ComparisonOperator::Less,
                        Box::new(node),
                        Box::new(self.parse_sum()?),
                    )
                }
                TokenType::LessEqual => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        ComparisonOperator::LessEqual,
                        Box::new(node),
                        Box::new(self.parse_sum()?),
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_sum(&mut self) -> Result<Node, Error> {
        let mut node = self.parse_product()?;

        while self.cursor < self.tokens.len() {
            match self.tokens[self.cursor].token_type {
                TokenType::Plus => {
                    self.cursor += 1;
                    node = Node::Sum(
                        SumOperator::Plus,
                        Box::new(node),
                        Box::new(self.parse_product()?),
                    )
                }
                TokenType::Minus => {
                    self.cursor += 1;
                    node = Node::Sum(
                        SumOperator::Minus,
                        Box::new(node),
                        Box::new(self.parse_product()?),
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_product(&mut self) -> Result<Node, Error> {
        let mut node = self.parse_unary()?;

        while self.cursor < self.tokens.len() {
            match self.tokens[self.cursor].token_type {
                TokenType::Star => {
                    self.cursor += 1;
                    node = Node::Product(
                        ProductOperator::Star,
                        Box::new(node),
                        Box::new(self.parse_unary()?),
                    )
                }
                TokenType::Slash => {
                    self.cursor += 1;
                    node = Node::Product(
                        ProductOperator::Slash,
                        Box::new(node),
                        Box::new(self.parse_unary()?),
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Node, Error> {
        let node = match self.tokens[self.cursor].token_type {
            TokenType::Bang => {
                self.cursor += 1;
                Node::Unary(UnaryOperator::Bang, Box::new(self.parse_unary()?))
            }
            TokenType::Minus => {
                self.cursor += 1;
                Node::Unary(UnaryOperator::Minus, Box::new(self.parse_unary()?))
            }
            _ => self.parse_primary()?,
        };

        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<Node, Error> {
        match self.tokens[self.cursor].token_type {
            TokenType::Nil => {
                self.cursor += 1;
                Ok(Node::Primary(Literal::Nil))
            }
            TokenType::True => {
                self.cursor += 1;
                Ok(Node::Primary(Literal::True))
            }
            TokenType::False => {
                self.cursor += 1;
                Ok(Node::Primary(Literal::False))
            }
            TokenType::Number => {
                if let Ok(number) = self.tokens[self.cursor].lexeme.parse::<f64>() {
                    self.cursor += 1;
                    Ok(Node::Primary(Literal::Number(number)))
                } else {
                    Err(Error::new(
                        self.tokens[self.cursor].line_number,
                        &format!(
                            "Failed to parse number: {:?}",
                            self.tokens[self.cursor].lexeme
                        ),
                    ))
                }
            }
            TokenType::String => {
                let string = self.tokens[self.cursor].lexeme.to_string();
                self.cursor += 1;
                Ok(Node::Primary(Literal::String(string)))
            }
            TokenType::LeftParen => {
                self.cursor += 1;
                let node = self.parse_expression()?;
                if self.cursor < self.tokens.len()
                    && self.tokens[self.cursor].token_type == TokenType::RightParen
                {
                    self.cursor += 1;
                    Ok(node)
                } else {
                    Err(Error::new(
                        self.tokens[self.cursor - 1].line_number,
                        &format!("Unexpected token: {}", self.tokens[self.cursor - 1].lexeme),
                    ))
                }
            }
            _ => Err(Error::new(
                self.tokens[self.cursor].line_number,
                &format!("Unexpected token: {}", self.tokens[self.cursor].lexeme),
            )),
        }
    }
}

impl<'a> Parse<'a> for RecursiveDescentParser<'a> {
    fn parse(&mut self, tokens: &'a [Token]) -> Result<Node, Error> {
        self.tokens = tokens;
        self.cursor = 0;

        let node = self.parse_expression()?;
        if self.cursor < self.tokens.len() {
            Err(Error::new(
                self.tokens[self.cursor].line_number,
                &format!("Unexpected token bruh: {}", self.tokens[self.cursor].lexeme),
            ))
        } else {
            Ok(node)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        ComparisonOperator, EqualityOperator, Literal, Node, ProductOperator, SumOperator,
        UnaryOperator,
    };
    use crate::parser::Parse;
    use crate::token::{Token, TokenType};

    use super::RecursiveDescentParser;

    struct TestCase<'a> {
        input: &'a [Token<'a>],
        expected_output: Node,
    }

    #[test]
    fn can_parse_series() {
        let input = &[
            Token::new(TokenType::Number, "1", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Number, "2", 1),
        ];
        let expected_output = Node::Expression(Box::new(Node::Series(
            Box::new(Node::Primary(Literal::Number(1.0))),
            Box::new(Node::Primary(Literal::Number(2.0))),
        )));
        let output = RecursiveDescentParser::new().parse(input).unwrap();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn series_is_left_associative() {
        let input = &[
            Token::new(TokenType::Number, "1", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Number, "2", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Number, "3", 1),
        ];
        let expected_output = Node::Expression(Box::new(Node::Series(
            Box::new(Node::Series(
                Box::new(Node::Primary(Literal::Number(1.0))),
                Box::new(Node::Primary(Literal::Number(2.0))),
            )),
            Box::new(Node::Primary(Literal::Number(3.0))),
        )));

        let output = RecursiveDescentParser::new().parse(input).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn can_parse_equality() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::EqualEqual, "==", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Equality(
                    EqualityOperator::Equal,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::BangEqual, "!=", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Equality(
                    EqualityOperator::NotEqual,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn equality_is_left_associative() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::EqualEqual, "==", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::EqualEqual, "==", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Equality(
                    EqualityOperator::Equal,
                    Box::new(Node::Equality(
                        EqualityOperator::Equal,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::BangEqual, "!=", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::BangEqual, "!=", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Equality(
                    EqualityOperator::NotEqual,
                    Box::new(Node::Equality(
                        EqualityOperator::NotEqual,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn can_parse_comparison() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Greater, ">", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::Greater,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::GreaterEqual, ">=", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::GreaterEqual,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Less, "<", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::Less,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::LessEqual, "<=", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::LessEqual,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn comparison_is_left_associative() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Greater, ">", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::Greater, ">", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::Greater,
                    Box::new(Node::Comparison(
                        ComparisonOperator::Greater,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::GreaterEqual, ">=", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::GreaterEqual, ">=", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::GreaterEqual,
                    Box::new(Node::Comparison(
                        ComparisonOperator::GreaterEqual,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Less, "<", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::Less, "<", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::Less,
                    Box::new(Node::Comparison(
                        ComparisonOperator::Less,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::LessEqual, "<=", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::LessEqual, "<=", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Comparison(
                    ComparisonOperator::LessEqual,
                    Box::new(Node::Comparison(
                        ComparisonOperator::LessEqual,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn can_parse_sum() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Plus, "+", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Sum(
                    SumOperator::Plus,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Minus, "-", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Sum(
                    SumOperator::Minus,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn sum_is_left_associative() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Plus, "+", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::Plus, "+", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Sum(
                    SumOperator::Plus,
                    Box::new(Node::Sum(
                        SumOperator::Plus,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Minus, "-", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::Minus, "-", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Sum(
                    SumOperator::Minus,
                    Box::new(Node::Sum(
                        SumOperator::Minus,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn product_is_left_associative() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Star, "*", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::Star, "*", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Product(
                    ProductOperator::Star,
                    Box::new(Node::Product(
                        ProductOperator::Star,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Slash, "/", 1),
                    Token::new(TokenType::Number, "2", 1),
                    Token::new(TokenType::Slash, "/", 1),
                    Token::new(TokenType::Number, "3", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Product(
                    ProductOperator::Slash,
                    Box::new(Node::Product(
                        ProductOperator::Slash,
                        Box::new(Node::Primary(Literal::Number(1.0))),
                        Box::new(Node::Primary(Literal::Number(2.0))),
                    )),
                    Box::new(Node::Primary(Literal::Number(3.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn can_parse_product() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Star, "*", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Product(
                    ProductOperator::Star,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Number, "1", 1),
                    Token::new(TokenType::Slash, "/", 1),
                    Token::new(TokenType::Number, "2", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Product(
                    ProductOperator::Slash,
                    Box::new(Node::Primary(Literal::Number(1.0))),
                    Box::new(Node::Primary(Literal::Number(2.0))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn can_parse_unary() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Bang, "!", 1),
                    Token::new(TokenType::True, "true", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Unary(
                    UnaryOperator::Bang,
                    Box::new(Node::Primary(Literal::True)),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Minus, "-", 1),
                    Token::new(TokenType::Number, "123.456", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Unary(
                    UnaryOperator::Minus,
                    Box::new(Node::Primary(Literal::Number(123.456))),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn unary_is_right_associative() {
        let test_cases = [
            TestCase {
                input: &[
                    Token::new(TokenType::Bang, "!", 1),
                    Token::new(TokenType::Bang, "!", 1),
                    Token::new(TokenType::True, "true", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Unary(
                    UnaryOperator::Bang,
                    Box::new(Node::Unary(
                        UnaryOperator::Bang,
                        Box::new(Node::Primary(Literal::True)),
                    )),
                ))),
            },
            TestCase {
                input: &[
                    Token::new(TokenType::Minus, "-", 1),
                    Token::new(TokenType::Minus, "-", 1),
                    Token::new(TokenType::Number, "123.456", 1),
                ],
                expected_output: Node::Expression(Box::new(Node::Unary(
                    UnaryOperator::Minus,
                    Box::new(Node::Unary(
                        UnaryOperator::Minus,
                        Box::new(Node::Primary(Literal::Number(123.456))),
                    )),
                ))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn can_parse_keywords() {
        let test_cases = [
            TestCase {
                input: &[Token::new(TokenType::Nil, "nil", 1)],
                expected_output: Node::Expression(Box::new(Node::Primary(Literal::Nil))),
            },
            TestCase {
                input: &[Token::new(TokenType::True, "true", 1)],
                expected_output: Node::Expression(Box::new(Node::Primary(Literal::True))),
            },
            TestCase {
                input: &[Token::new(TokenType::False, "false", 1)],
                expected_output: Node::Expression(Box::new(Node::Primary(Literal::False))),
            },
        ];

        for test_case in test_cases {
            let output = RecursiveDescentParser::new()
                .parse(test_case.input)
                .unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }

    #[test]
    fn can_parse_string_literal() {
        let input = &[Token::new(TokenType::String, "I am a string!", 1)];
        let expected_output = Node::Expression(Box::new(Node::Primary(Literal::String(
            String::from("I am a string!"),
        ))));
        let output = RecursiveDescentParser::new().parse(input).unwrap();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn can_parse_number_literal() {
        let input = &[Token::new(TokenType::Number, "123.456", 1)];
        let expected_output = Node::Expression(Box::new(Node::Primary(Literal::Number(123.456))));
        let output = RecursiveDescentParser::new().parse(input).unwrap();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn can_parse_parenthetical_expression() {
        let input = &[
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::Nil, "nil", 1),
            Token::new(TokenType::RightParen, ")", 1),
        ];
        let expected_output = Node::Expression(Box::new(Node::Expression(Box::new(
            Node::Primary(Literal::Nil),
        ))));
        let output = RecursiveDescentParser::new().parse(input).unwrap();

        assert_eq!(output, expected_output);
    }
}
