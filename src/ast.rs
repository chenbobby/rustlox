#[derive(Debug, PartialEq)]
pub enum Node {
    Expression(Box<Node>),
    Series(Box<Node>, Box<Node>),
    Equality(EqualityOperator, Box<Node>, Box<Node>),
    Comparison(ComparisonOperator, Box<Node>, Box<Node>),
    Sum(SumOperator, Box<Node>, Box<Node>),
    Product(ProductOperator, Box<Node>, Box<Node>),
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
