use crate::scanner::{Token, TokenType};

#[derive(Debug)]
pub enum EqualityOperator {
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub enum ComparisonOperator {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug)]
pub enum SumOperator {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum ProductOperator {
    Star,
    Slash,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug)]
pub enum Literal {
    Nil,
    True,
    False,
    Number(f64),
    String(String),
    Grouping(Box<Node>),
}

#[derive(Debug)]
pub enum Node {
    Expression(Box<Node>),
    Equality(Box<Node>, EqualityOperator, Box<Node>),
    Comparison(Box<Node>, ComparisonOperator, Box<Node>),
    Sum(Box<Node>, SumOperator, Box<Node>),
    Product(Box<Node>, ProductOperator, Box<Node>),
    Unary(UnaryOperator, Box<Node>),
    Primary(Literal),
}

pub struct RecursiveDescentParser<'a> {
    cursor: usize,
    tokens: &'a [Token],
    tokens_len: usize,
}

impl<'a> RecursiveDescentParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        RecursiveDescentParser {
            cursor: 0,
            tokens: tokens,
            tokens_len: tokens.len(),
        }
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        self.cursor = 0;
        let node = self.parse_expression()?;
        Ok(node)
    }

    fn parse_expression(&mut self) -> Result<Node, String> {
        let equality = Node::Expression(Box::new(self.parse_equality()?));
        Ok(equality)
    }

    fn parse_equality(&mut self) -> Result<Node, String> {
        let mut node = self.parse_comparison()?;

        while self.cursor < self.tokens_len {
            match self.tokens[self.cursor].token_type {
                TokenType::EqualEqual => {
                    self.cursor += 1;
                    node = Node::Equality(
                        Box::new(node),
                        EqualityOperator::Equal,
                        Box::new(self.parse_comparison()?),
                    );
                }
                TokenType::BangEqual => {
                    self.cursor += 1;
                    node = Node::Equality(
                        Box::new(node),
                        EqualityOperator::NotEqual,
                        Box::new(self.parse_comparison()?),
                    );
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<Node, String> {
        let mut node = self.parse_sum()?;

        while self.cursor < self.tokens_len {
            match self.tokens[self.cursor].token_type {
                TokenType::Greater => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        Box::new(node),
                        ComparisonOperator::Greater,
                        Box::new(self.parse_sum()?),
                    )
                }
                TokenType::GreaterEqual => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        Box::new(node),
                        ComparisonOperator::GreaterEqual,
                        Box::new(self.parse_sum()?),
                    )
                }
                TokenType::Less => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        Box::new(node),
                        ComparisonOperator::Less,
                        Box::new(self.parse_sum()?),
                    )
                }
                TokenType::LessEqual => {
                    self.cursor += 1;
                    node = Node::Comparison(
                        Box::new(node),
                        ComparisonOperator::LessEqual,
                        Box::new(self.parse_sum()?),
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_sum(&mut self) -> Result<Node, String> {
        let mut node = self.parse_product()?;

        while self.cursor < self.tokens_len {
            match self.tokens[self.cursor].token_type {
                TokenType::Plus => {
                    self.cursor += 1;
                    node = Node::Sum(
                        Box::new(node),
                        SumOperator::Plus,
                        Box::new(self.parse_product()?),
                    )
                }
                TokenType::Minus => {
                    self.cursor += 1;
                    node = Node::Sum(
                        Box::new(node),
                        SumOperator::Minus,
                        Box::new(self.parse_product()?),
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_product(&mut self) -> Result<Node, String> {
        let mut node = self.parse_unary()?;

        while self.cursor < self.tokens_len {
            match self.tokens[self.cursor].token_type {
                TokenType::Star => {
                    self.cursor += 1;
                    node = Node::Product(
                        Box::new(node),
                        ProductOperator::Star,
                        Box::new(self.parse_unary()?),
                    )
                }
                TokenType::Minus => {
                    self.cursor += 1;
                    node = Node::Product(
                        Box::new(node),
                        ProductOperator::Slash,
                        Box::new(self.parse_unary()?),
                    )
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Node, String> {
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

    fn parse_primary(&mut self) -> Result<Node, String> {
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
                    Err(format!(
                        "Failed to parse number: {:?}",
                        self.tokens[self.cursor].lexeme
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
                if self.cursor < self.tokens_len
                    && self.tokens[self.cursor].token_type == TokenType::RightParen
                {
                    self.cursor += 1;
                    Ok(Node::Primary(Literal::Grouping(Box::new(node))))
                } else {
                    Err(format!(
                        "Expected token ')' at line {}",
                        self.tokens[self.cursor - 1].line_number,
                    ))
                }
            }
            _ => Err(format!(
                "Unexpected token at line {}: {:?}",
                self.tokens[self.cursor].line_number, self.tokens[self.cursor].lexeme,
            )),
        }
    }
}
