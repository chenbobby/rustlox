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
