use std::io::Write; // Brings std::io::Stdout.flush into scope.

const EXIT_CODE_USAGE: i32 = 32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: rustlox [script]");
        std::process::exit(EXIT_CODE_USAGE);
    } else if args.len() == 2 {
        let filename = &args[1];
        let mut interpreter = Interpreter::new();
        if let Err(error) = interpreter.run_file(filename) {
            eprintln!("{}", error);
            let code = error.raw_os_error().unwrap_or(1);
            std::process::exit(code);
        }
    } else {
        let mut interpreter = Interpreter::new();
        if let Err(error) = interpreter.run_prompt() {
            eprintln!("{}", error);
            let code = error.raw_os_error().unwrap_or(1);
            std::process::exit(code);
        }
    }
}

struct Interpreter {
    had_error: bool,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter { had_error: false }
    }

    fn run_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let mut source_code = std::fs::read_to_string(filename)?;
        self.run(&mut source_code);
        Ok(())
    }

    fn run_prompt(&mut self) -> Result<(), std::io::Error> {
        loop {
            print!(">> ");
            std::io::stdout().flush()?;
            let mut source_code = String::new();
            match std::io::stdin().read_line(&mut source_code) {
                Err(error) => return Err(error),
                Ok(0) => return Ok(()),
                Ok(_) => {
                    self.run(&mut source_code);
                    if self.had_error {
                        return Ok(());
                    }
                }
            };
        }
    }

    fn run(&mut self, source_code: &mut str) {
        let mut scanner = Scanner::new();
        if let Err((line_number, message)) = scanner.scan(source_code) {
            self.error(line_number, &message)
        }
        let tokens = &scanner.tokens;
        println!("Token:\n{:#?}", tokens);
        let node = ast::RecursiveDescentParser::new(tokens).parse();
        println!("AST:\n{:#?}", node);
    }

    fn error(&mut self, line_number: i32, message: &str) {
        self.report(line_number, message);
    }

    fn report(&mut self, line_number: i32, message: &str) {
        eprintln!("[line {}] Error: {}", line_number, message);
        self.had_error = true;
    }
}

struct Scanner {
    cursor: usize,
    lexeme_start: usize,
    line_number: i32,
    tokens: Vec<Token>,
}

impl Scanner {
    fn new() -> Scanner {
        Scanner {
            cursor: 0,
            lexeme_start: 0,
            line_number: 1,
            tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token_type: TokenType, lexeme: &str) {
        let token = Token::new(token_type, lexeme, self.line_number);
        self.tokens.push(token);
        self.cursor += 1;
        self.lexeme_start = self.cursor;
    }

    // [TODO]: Improve the API of Scanner.scan so that it returns a Vec<Token> without re-allocating
    // the entire backing array.
    fn scan(&mut self, source_code: &mut str) -> Result<(), (i32, String)> {
        self.tokens = Vec::new();

        let characters: Vec<char> = source_code.chars().collect();
        'scan: while self.cursor < characters.len() {
            match characters[self.cursor] {
                '(' => self.add_token(TokenType::LeftParen, "("),
                ')' => self.add_token(TokenType::RightParen, ")"),
                '{' => self.add_token(TokenType::LeftBrace, "{"),
                '}' => self.add_token(TokenType::RightBrace, "}"),
                ';' => self.add_token(TokenType::Semicolon, ";"),
                ',' => self.add_token(TokenType::Comma, ","),
                '.' => self.add_token(TokenType::Dot, "."),
                '+' => self.add_token(TokenType::Plus, "+"),
                '-' => self.add_token(TokenType::Minus, "-"),
                '*' => self.add_token(TokenType::Star, "*"),
                '/' => {
                    if characters[self.cursor + 1] == '/' {
                        'line_comment: loop {
                            self.cursor += 1;
                            if self.cursor >= characters.len() || characters[self.cursor] == '\n' {
                                self.cursor += 1;
                                self.lexeme_start = self.cursor;
                                self.line_number += 1;
                                break 'line_comment;
                            }
                        }
                    } else {
                        self.add_token(TokenType::Slash, "/")
                    }
                }
                '!' => {
                    if characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::BangEqual, "!=")
                    } else {
                        self.add_token(TokenType::Bang, "!")
                    }
                }
                '=' => {
                    if characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::EqualEqual, "==")
                    } else {
                        self.add_token(TokenType::Equal, "=")
                    }
                }
                '>' => {
                    if characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::GreaterEqual, ">=")
                    } else {
                        self.add_token(TokenType::Greater, ">")
                    }
                }
                '<' => {
                    if characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::LessEqual, "<=")
                    } else {
                        self.add_token(TokenType::Less, "<")
                    }
                }
                '"' => 'string_literal: loop {
                    self.cursor += 1;

                    if characters[self.cursor] == '"' {
                        let slice = &characters[self.lexeme_start + 1..self.cursor];
                        let string_literal = &String::from_iter(slice);
                        self.add_token(TokenType::String, string_literal);
                        break 'string_literal;
                    }

                    if self.cursor >= characters.len() || characters[self.cursor] == '\n' {
                        return Err((self.line_number, "Unterminated string".to_string()));
                    }
                },
                ' ' | '\r' | '\t' => {
                    // Ignore whitespace.
                    self.cursor += 1;
                    self.lexeme_start = self.cursor;
                }
                '\n' => {
                    self.cursor += 1;
                    self.lexeme_start = self.cursor;
                    self.line_number += 1;
                }
                _ => {
                    if characters[self.cursor].is_digit(10) {
                        'number_literal: loop {
                            self.cursor += 1;

                            if characters[self.cursor].is_digit(10) {
                                continue 'number_literal;
                            }

                            if characters[self.cursor] == '.'
                                && characters[self.cursor + 1].is_digit(10)
                            {
                                continue 'number_literal;
                            }

                            self.cursor -= 1;
                            let slice = &characters[self.lexeme_start..self.cursor + 1];
                            let number_literal = &String::from_iter(slice);
                            self.add_token(TokenType::Number, number_literal);
                            break 'number_literal;
                        }
                        continue 'scan;
                    }

                    if characters[self.cursor].is_alphabetic() || characters[self.cursor] == '_' {
                        'identifier_literal: loop {
                            self.cursor += 1;

                            if characters[self.cursor].is_alphanumeric()
                                || characters[self.cursor] == '_'
                            {
                                continue 'identifier_literal;
                            }

                            let slice = &characters[self.lexeme_start..self.cursor];
                            let identifier_literal = &String::from_iter(slice);
                            let token_type = TokenType::from_str(identifier_literal);
                            self.add_token(token_type, identifier_literal);
                            break 'identifier_literal;
                        }
                        continue 'scan;
                    }

                    return Err((
                        self.line_number,
                        format!("Unexpected character: {}", characters[self.cursor]),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,

    // Multi-character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    Nil,
    True,
    False,
    And,
    Or,
    If,
    Else,
    For,
    While,
    Var,
    Fun,
    Return,
    Class,
    This,
    Super,
    Print,
}

impl TokenType {
    fn from_str(identifier_literal: &str) -> Self {
        match identifier_literal {
            "nil" => TokenType::Nil,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "var" => TokenType::Var,
            "fun" => TokenType::Fun,
            "return" => TokenType::Return,
            "class" => TokenType::Class,
            "this" => TokenType::This,
            "super" => TokenType::Super,
            "print" => TokenType::Print,
            _ => TokenType::Identifier,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line_number: i32,
}

impl Token {
    fn new(token_type: TokenType, lexeme: &str, line_number: i32) -> Token {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            line_number,
        }
    }
}

mod ast {
    use super::{Token, TokenType};

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
}
