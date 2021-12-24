#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Tokens with only 1 character
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

    // Tokens with at least 1 character
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

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line_number: i32,
}

impl<'a> Token<'a> {
    fn new(token_type: TokenType, lexeme: &'a str, line_number: i32) -> Token {
        Token {
            token_type,
            lexeme: lexeme,
            line_number,
        }
    }
}

pub struct Scanner<'a> {
    cursor: usize,
    lexeme_start: usize,
    line_number: i32,
    pub tokens: Vec<Token<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new() -> Scanner<'a> {
        Scanner {
            cursor: 0,
            lexeme_start: 0,
            line_number: 1,
            tokens: Vec::new(),
        }
    }

    // [TODO]: Improve the API of Scanner.scan so that it returns a Vec<Token> without re-allocating
    // the entire backing array.
    pub fn scan(&mut self, source_code: &'a str) -> Result<Vec<Token<'a>>, Error> {
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
                        let string_literal = &source_code[self.lexeme_start + 1..self.cursor];
                        self.add_token(TokenType::String, string_literal);
                        break 'string_literal;
                    }

                    if self.cursor >= characters.len() || characters[self.cursor] == '\n' {
                        return Err(Error::new(self.line_number, "Unterminated string"));
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
                            let number_literal = &source_code[self.lexeme_start..self.cursor + 1];
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

                            let identifier_literal = &source_code[self.lexeme_start..self.cursor];
                            let token_type = TokenType::from_str(identifier_literal);
                            self.add_token(token_type, identifier_literal);
                            break 'identifier_literal;
                        }
                        continue 'scan;
                    }

                    return Err(Error::new(
                        self.line_number,
                        &format!("Unexpected character: {}", characters[self.cursor]),
                    ));
                }
            }
        }

        Ok(self.tokens[..].to_vec())
    }

    pub fn add_token(&mut self, token_type: TokenType, lexeme: &'a str) {
        let token = Token::new(token_type, lexeme, self.line_number);
        self.tokens.push(token);
        self.cursor += 1;
        self.lexeme_start = self.cursor;
    }
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

#[cfg(test)]
mod tests {
    use crate::scanner::{Scanner, Token, TokenType};

    #[test]
    fn single_character_lexemes_are_scanned_successfully() {
        struct TestCase<'a> {
            input: &'a str,
            expected_output: Vec<Token<'a>>,
        }

        let test_cases = vec![
            TestCase {
                input: "(",
                expected_output: vec![Token {
                    token_type: TokenType::LeftParen,
                    lexeme: "(",
                    line_number: 1,
                }],
            },
            TestCase {
                input: ")",
                expected_output: vec![Token {
                    token_type: TokenType::RightParen,
                    lexeme: ")",
                    line_number: 1,
                }],
            },
        ];

        for test_case in test_cases {
            let mut scanner = Scanner::new();
            let output = scanner.scan(test_case.input).unwrap();

            assert_eq!(output, test_case.expected_output);
        }
    }
}
