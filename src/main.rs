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
        let tokens = scanner.tokens;
        println!("{:#?}", tokens);
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

        self.add_token(TokenType::EOF, "");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
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

    // End-of-file
    EOF,
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
struct Token {
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
