#[derive(Debug, PartialEq)]
pub enum Node {
    Expression(Box<Node>),
    Equality(Box<Node>, EqualityOperator, Box<Node>),
    Comparison(Box<Node>, ComparisonOperator, Box<Node>),
    Sum(Box<Node>, SumOperator, Box<Node>),
    Product(Box<Node>, ProductOperator, Box<Node>),
    Unary(UnaryOperator, Box<Node>),
    Primary(Literal),
}

#[derive(Debug, PartialEq)]
pub enum EqualityOperator {
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOperator {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, PartialEq)]
pub enum SumOperator {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum ProductOperator {
    Star,
    Slash,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Nil,
    True,
    False,
    String(String),
    Number(f64),
}
