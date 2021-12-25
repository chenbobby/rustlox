use crate::token::{Token, TokenType};

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
                    if self.cursor + 1 < characters.len() && characters[self.cursor + 1] == '/' {
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
                    if self.cursor + 1 < characters.len() && characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::BangEqual, "!=")
                    } else {
                        self.add_token(TokenType::Bang, "!")
                    }
                }
                '=' => {
                    if self.cursor + 1 < characters.len() && characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::EqualEqual, "==")
                    } else {
                        self.add_token(TokenType::Equal, "=")
                    }
                }
                '>' => {
                    if self.cursor + 1 < characters.len() && characters[self.cursor + 1] == '=' {
                        self.cursor += 1;
                        self.add_token(TokenType::GreaterEqual, ">=")
                    } else {
                        self.add_token(TokenType::Greater, ">")
                    }
                }
                '<' => {
                    if self.cursor + 1 < characters.len() && characters[self.cursor + 1] == '=' {
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

                            if self.cursor < characters.len()
                                && characters[self.cursor].is_digit(10)
                            {
                                continue 'number_literal;
                            }

                            if self.cursor < characters.len() && characters[self.cursor] == '.' {
                                'number_decimal_loop: loop {
                                    self.cursor += 1;

                                    if self.cursor < characters.len()
                                        && characters[self.cursor].is_digit(10)
                                    {
                                        continue 'number_decimal_loop;
                                    }

                                    break 'number_decimal_loop;
                                }
                            }

                            break 'number_literal;
                        }

                        self.cursor -= 1;
                        let number_literal = &source_code[self.lexeme_start..self.cursor + 1];
                        self.add_token(TokenType::Number, number_literal);

                        continue 'scan;
                    }

                    if characters[self.cursor].is_alphabetic() || characters[self.cursor] == '_' {
                        'identifier_literal: loop {
                            self.cursor += 1;

                            if self.cursor < characters.len()
                                && (characters[self.cursor].is_alphanumeric()
                                    || characters[self.cursor] == '_')
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
    fn lexemes_are_scanned_successfully() {
        struct TestCase<'a> {
            input: &'a str,
            expected_output: Vec<Token<'a>>,
        }

        let test_cases = vec![
            TestCase {
                input: "(",
                expected_output: vec![Token::new(TokenType::LeftParen, "(", 1)],
            },
            TestCase {
                input: ")",
                expected_output: vec![Token::new(TokenType::RightParen, ")", 1)],
            },
            TestCase {
                input: "{",
                expected_output: vec![Token::new(TokenType::LeftBrace, "{", 1)],
            },
            TestCase {
                input: "}",
                expected_output: vec![Token::new(TokenType::RightBrace, "}", 1)],
            },
            TestCase {
                input: ";",
                expected_output: vec![Token::new(TokenType::Semicolon, ";", 1)],
            },
            TestCase {
                input: ",",
                expected_output: vec![Token::new(TokenType::Comma, ",", 1)],
            },
            TestCase {
                input: ".",
                expected_output: vec![Token::new(TokenType::Dot, ".", 1)],
            },
            TestCase {
                input: "+",
                expected_output: vec![Token::new(TokenType::Plus, "+", 1)],
            },
            TestCase {
                input: "-",
                expected_output: vec![Token::new(TokenType::Minus, "-", 1)],
            },
            TestCase {
                input: "*",
                expected_output: vec![Token::new(TokenType::Star, "*", 1)],
            },
            TestCase {
                input: "/",
                expected_output: vec![Token::new(TokenType::Slash, "/", 1)],
            },
            TestCase {
                input: "!",
                expected_output: vec![Token::new(TokenType::Bang, "!", 1)],
            },
            TestCase {
                input: "!=",
                expected_output: vec![Token::new(TokenType::BangEqual, "!=", 1)],
            },
            TestCase {
                input: "=",
                expected_output: vec![Token::new(TokenType::Equal, "=", 1)],
            },
            TestCase {
                input: "==",
                expected_output: vec![Token::new(TokenType::EqualEqual, "==", 1)],
            },
            TestCase {
                input: ">",
                expected_output: vec![Token::new(TokenType::Greater, ">", 1)],
            },
            TestCase {
                input: ">=",
                expected_output: vec![Token::new(TokenType::GreaterEqual, ">=", 1)],
            },
            TestCase {
                input: "<",
                expected_output: vec![Token::new(TokenType::Less, "<", 1)],
            },
            TestCase {
                input: "<=",
                expected_output: vec![Token::new(TokenType::LessEqual, "<=", 1)],
            },
            TestCase {
                input: "nil",
                expected_output: vec![Token::new(TokenType::Nil, "nil", 1)],
            },
            TestCase {
                input: "true",
                expected_output: vec![Token::new(TokenType::True, "true", 1)],
            },
            TestCase {
                input: "false",
                expected_output: vec![Token::new(TokenType::False, "false", 1)],
            },
            TestCase {
                input: "and",
                expected_output: vec![Token::new(TokenType::And, "and", 1)],
            },
            TestCase {
                input: "or",
                expected_output: vec![Token::new(TokenType::Or, "or", 1)],
            },
            TestCase {
                input: "if",
                expected_output: vec![Token::new(TokenType::If, "if", 1)],
            },
            TestCase {
                input: "else",
                expected_output: vec![Token::new(TokenType::Else, "else", 1)],
            },
            TestCase {
                input: "for",
                expected_output: vec![Token::new(TokenType::For, "for", 1)],
            },
            TestCase {
                input: "while",
                expected_output: vec![Token::new(TokenType::While, "while", 1)],
            },
            TestCase {
                input: "var",
                expected_output: vec![Token::new(TokenType::Var, "var", 1)],
            },
            TestCase {
                input: "fun",
                expected_output: vec![Token::new(TokenType::Fun, "fun", 1)],
            },
            TestCase {
                input: "return",
                expected_output: vec![Token::new(TokenType::Return, "return", 1)],
            },
            TestCase {
                input: "class",
                expected_output: vec![Token::new(TokenType::Class, "class", 1)],
            },
            TestCase {
                input: "this",
                expected_output: vec![Token::new(TokenType::This, "this", 1)],
            },
            TestCase {
                input: "super",
                expected_output: vec![Token::new(TokenType::Super, "super", 1)],
            },
            TestCase {
                input: "print",
                expected_output: vec![Token::new(TokenType::Print, "print", 1)],
            },
            TestCase {
                input: "\"sushi\"",
                expected_output: vec![Token::new(TokenType::String, "sushi", 1)],
            },
            TestCase {
                input: "123.456",
                expected_output: vec![Token::new(TokenType::Number, "123.456", 1)],
            },
            TestCase {
                input: "my_variable",
                expected_output: vec![Token::new(TokenType::Identifier, "my_variable", 1)],
            },
        ];

        for test_case in test_cases {
            let output = Scanner::new().scan(test_case.input).unwrap();
            assert_eq!(output, test_case.expected_output);
        }
    }
}
